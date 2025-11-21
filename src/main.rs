// main.rs
mod framebuffer;
mod triangle;
mod obj;
mod matrix;
mod fragment;
mod vertex;
mod camera;
mod shaders;
mod light;
mod line;

use framebuffer::Framebuffer;
use triangle::triangle;
use obj::Obj;
use raylib::prelude::*;
use std::thread;
use std::time::Duration;
use std::f32::consts::PI;
use matrix::{create_model_matrix, create_projection_matrix, create_viewport_matrix, create_view_matrix, multiply_matrix_vector4};
use vertex::Vertex;
use camera::Camera;
use shaders::{vertex_shader, fragment_shader, mercury_fragment_shader, sun_fragment_shader, earth_fragment_shader, mars_fragment_shader, uranus_fragment_shader, nave_fragment_shader, skybox_fragment_shader};
use light::Light;

/// Helpers para operar con `raylib::prelude::Vector3` de forma segura
fn add_vec3(a: Vector3, b: Vector3) -> Vector3 {
    Vector3::new(a.x + b.x, a.y + b.y, a.z + b.z)
}
fn sub_vec3(a: Vector3, b: Vector3) -> Vector3 {
    Vector3::new(a.x - b.x, a.y - b.y, a.z - b.z)
}
fn mul_vec3_scalar(v: Vector3, s: f32) -> Vector3 {
    Vector3::new(v.x * s, v.y * s, v.z * s)
}
fn length_vec3(v: Vector3) -> f32 {
    (v.x * v.x + v.y * v.y + v.z * v.z).sqrt()
}
fn normalize_vec3(mut v: Vector3) -> Vector3 {
    let len = length_vec3(v);
    if len != 0.0 {
        v.x /= len;
        v.y /= len;
        v.z /= len;
    }
    v
}
fn clamp_f32(x: f32, lo: f32, hi: f32) -> f32 {
    if x < lo { lo } else if x > hi { hi } else { x }
}

pub struct Uniforms {
    pub model_matrix: Matrix,
    pub view_matrix: Matrix,
    pub projection_matrix: Matrix,
    pub viewport_matrix: Matrix,
    pub time: f32,
    pub dt: f32,
}

fn render(
    framebuffer: &mut Framebuffer,
    uniforms: &Uniforms,
    vertex_array: &[Vertex],
    light: &Light,
    planet_type: &str,
) {
    let mut transformed_vertices = Vec::with_capacity(vertex_array.len());
    for vertex in vertex_array {
        transformed_vertices.push(vertex_shader(vertex, uniforms));
    }
    let mut triangles = Vec::new();
    for i in (0..transformed_vertices.len()).step_by(3) {
        if i + 2 < transformed_vertices.len() {
            triangles.push([
                transformed_vertices[i].clone(),
                transformed_vertices[i + 1].clone(),
                transformed_vertices[i + 2].clone(),
            ]);
        }
    }
    let mut fragments = Vec::new();
    for tri in &triangles {
        fragments.extend(triangle(&tri[0], &tri[1], &tri[2], light));
    }
    for fragment in fragments {
        // Protección: evitar NaN/Inf y fragmentos fuera de pantalla para prevenir panics/overflows
        if !fragment.position.x.is_finite() || !fragment.position.y.is_finite() || !fragment.depth.is_finite() {
            continue;
        }
        let sx = fragment.position.x.round() as i32;
        let sy = fragment.position.y.round() as i32;
        if sx < 0 || sx >= framebuffer.width || sy < 0 || sy >= framebuffer.height {
            continue;
        }

        let final_color = match planet_type {
            "Sun" => sun_fragment_shader(&fragment, uniforms),
            "Mercury" => mercury_fragment_shader(&fragment, uniforms),
            "Earth" => earth_fragment_shader(&fragment, uniforms),
            "Mars" => mars_fragment_shader(&fragment, uniforms),
            "Uranus" => uranus_fragment_shader(&fragment, uniforms),
            "Nave" => nave_fragment_shader(&fragment, uniforms),
            "Skybox" => skybox_fragment_shader(&fragment, uniforms),
            _ => fragment_shader(&fragment, uniforms),
        };
        framebuffer.point(
            sx,
            sy,
            final_color,
            fragment.depth,
        );
    }
}

// 🌟 Renderiza estrellas en el fondo (skybox simple)
fn render_skybox(framebuffer: &mut Framebuffer, view_matrix: &Matrix, projection_matrix: &Matrix, viewport_matrix: &Matrix, time: f32) {
    // Reducido a 200 estrellas para aligerar carga y reducir posibilidad de saturar fragment buffer
    let mut rng = fastrand::Rng::with_seed(time as u64);
    for _ in 0..200 {
        // Distribución esférica uniforme con radio grande
        let radius = 200.0_f32;
        let u = rng.f32();
        let v = rng.f32();
        let theta = 2.0_f32 * PI * u;
        let phi = (2.0_f32 * v - 1.0_f32).acos();
        let x = radius * phi.sin() * theta.cos();
        let y = radius * phi.cos();
        let z = radius * phi.sin() * theta.sin();

        // Proyectar punto 3D → pantalla
        let pos4 = Vector4::new(x, y, z, 1.0_f32);
        let view_pos = multiply_matrix_vector4(view_matrix, &pos4);
        let clip_pos = multiply_matrix_vector4(projection_matrix, &view_pos);
        if clip_pos.w == 0.0 { continue; }
        let ndc = Vector3::new(
            clip_pos.x / clip_pos.w,
            clip_pos.y / clip_pos.w,
            clip_pos.z / clip_pos.w,
        );
        let ndc4 = Vector4::new(ndc.x, ndc.y, ndc.z, 1.0_f32);
        let screen_pos = multiply_matrix_vector4(viewport_matrix, &ndc4);
        let sx = screen_pos.x as i32;
        let sy = screen_pos.y as i32;

        // Dibujar punto si está en pantalla
        if sx >= 0 && sx < framebuffer.width && sy >= 0 && sy < framebuffer.height {
            // Brillo variable: algunas estrellas más brillantes
            let brightness = 0.8_f32 + rng.f32() * 0.4_f32;
            let star_color = Vector3::new(brightness, brightness, brightness);
            framebuffer.point(sx, sy, star_color, clip_pos.z / clip_pos.w); // profundidad grande
        }
    }
}

fn draw_orbit_3d(framebuffer: &mut Framebuffer, orbit_radius: f32, orbit_color: Color, view_matrix: &Matrix, projection_matrix: &Matrix, viewport_matrix: &Matrix) {
    let segments = 128;
    let angle_increment = 2.0_f32 * PI / segments as f32;
    let mut prev_x = 0;
    let mut prev_y = 0;
    let mut first_point = true;
    let mut first_x = 0;
    let mut first_y = 0;
    for i in 0..segments {
        let angle = i as f32 * angle_increment;
        let x = angle.cos() * orbit_radius;
        let y = 0.0_f32;
        let z = angle.sin() * orbit_radius;
        let position_vec4 = Vector4::new(x, y, z, 1.0_f32);
        let view_position = multiply_matrix_vector4(view_matrix, &position_vec4);
        let clip_position = multiply_matrix_vector4(projection_matrix, &view_position);
        let ndc = if clip_position.w != 0.0 {
            Vector3::new(clip_position.x / clip_position.w, clip_position.y / clip_position.w, clip_position.z / clip_position.w)
        } else {
            Vector3::new(clip_position.x, clip_position.y, clip_position.z)
        };
        let ndc_vec4 = Vector4::new(ndc.x, ndc.y, ndc.z, 1.0_f32);
        let screen_position = multiply_matrix_vector4(viewport_matrix, &ndc_vec4);
        let screen_x = screen_position.x as i32;
        let screen_y = screen_position.y as i32;
        if i == 0 {
            first_x = screen_x;
            first_y = screen_y;
        }
        if !first_point {
            framebuffer.draw_line_with_depth(prev_x, prev_y, screen_x, screen_y, orbit_color, 1000.0_f32);
        } else {
            first_point = false;
        }
        prev_x = screen_x;
        prev_y = screen_y;
    }
    if segments > 0 {
        framebuffer.draw_line_with_depth(prev_x, prev_y, first_x, first_y, orbit_color, 1000.0_f32);
    }
}

#[derive(Clone)]
struct CelestialBody {
    name: String,
    translation: Vector3,
    scale: f32,
    rotation: Vector3,
    orbit_radius: f32,
    orbit_speed: f32,
    rotation_speed: f32,
    color: Color,
}

fn check_collision(pos1: Vector3, radius1: f32, pos2: Vector3, radius2: f32) -> bool {
    let dx = pos1.x - pos2.x;
    let dy = pos1.y - pos2.y;
    let dz = pos1.z - pos2.z;
    let dist_sq = dx*dx + dy*dy + dz*dz;
    dist_sq < (radius1 + radius2)*(radius1 + radius2)
}

fn avoid_collision(camera_pos: Vector3, target_pos: Vector3, celestial_bodies: &[CelestialBody], time: f32) -> (Vector3, Vector3) {
    let mut new_camera_pos = camera_pos;
    let mut new_target_pos = target_pos;
    for body in celestial_bodies {
        let body_pos = if body.name != "Sun" {
            let x = (time * body.orbit_speed).cos() * body.orbit_radius;
            let z = (time * body.orbit_speed).sin() * body.orbit_radius;
            Vector3::new(x, 0.0_f32, z)
        } else {
            body.translation
        };
        let camera_radius = 2.0_f32;
        let body_radius = body.scale * 0.8_f32;
        if check_collision(new_camera_pos, camera_radius, body_pos, body_radius) {
            let dx = new_camera_pos.x - body_pos.x;
            let dy = new_camera_pos.y - body_pos.y;
            let dz = new_camera_pos.z - body_pos.z;
            let dist = (dx*dx + dy*dy + dz*dz).sqrt();
            if dist > 0.0 {
                let min_dist = body_radius + camera_radius;
                let scale = min_dist / dist;
                new_camera_pos.x = body_pos.x + dx * scale;
                new_camera_pos.y = body_pos.y + dy * scale;
                new_camera_pos.z = body_pos.z + dz * scale;
            }
        }
        if check_collision(new_target_pos, camera_radius, body_pos, body_radius) {
            let dx = new_target_pos.x - body_pos.x;
            let dy = new_target_pos.y - body_pos.y;
            let dz = new_target_pos.z - body_pos.z;
            let dist = (dx*dx + dy*dy + dz*dz).sqrt();
            if dist > 0.0 {
                let min_dist = body_radius + camera_radius;
                let scale = min_dist / dist;
                new_target_pos.x = body_pos.x + dx * scale;
                new_target_pos.y = body_pos.y + dy * scale;
                new_target_pos.z = body_pos.z + dz * scale;
            }
        }
    }
    (new_camera_pos, new_target_pos)
}

// Estado para warping animado
#[derive(Clone)]
struct WarpTarget {
    eye: Vector3,
    target: Vector3,
    up: Vector3,
}

impl WarpTarget {
    fn to_camera_state(&self) -> Camera {
        Camera::new(self.eye, self.target, self.up)
    }
}

// 🌟 Nueva función: interpolar entre dos cámaras (Vector3)
fn lerp_vec3(a: Vector3, b: Vector3, t: f32) -> Vector3 {
    Vector3::new(
        a.x + (b.x - a.x) * t,
        a.y + (b.y - a.y) * t,
        a.z + (b.z - a.z) * t,
    )
}

// 🌟 Nueva función: ease-in-out cúbico
fn ease_in_out(t: f32) -> f32 {
    if t < 0.5 {
        4.0 * t * t * t
    } else {
        1.0 - (-2.0 * t + 2.0).powi(3) / 2.0
    }
}

fn main() {
    let window_width = 1300;
    let window_height = 900;
    let (mut window, raylib_thread) = raylib::init()
        .size(window_width, window_height)
        .title("Proyecto 3 - Sistema Solar")
        .log_level(TraceLogLevel::LOG_WARNING)
        .build();

    let mut framebuffer = Framebuffer::new(window_width, window_height);

    // Alejar la cámara para ver mejor todo el sistema
    let initial_camera_pos = Vector3::new(0.0_f32, 40.0_f32, 140.0_f32);
    let initial_camera_target = Vector3::new(0.0_f32, 0.0_f32, 0.0_f32);
    let initial_camera_up = Vector3::new(0.0_f32, 1.0_f32, 0.0_f32);
    let mut camera = Camera::new(initial_camera_pos, initial_camera_target, initial_camera_up);

    let light = Light::new(Vector3::new(0.0_f32, 0.0_f32, 0.0_f32));

    // Cargar nave y esfera (sphere como malla de planetas). Añadir logging y comprobación.
    let ship_obj = match Obj::load("./assets/nave.obj") {
        Ok(o) => {
            eprintln!("Loaded ./assets/nave.obj successfully");
            o
        },
        Err(e) => panic!("Failed to load ./assets/nave.obj: {}", e),
    };
    let nave_vertex_array = ship_obj.get_vertex_array();
    if nave_vertex_array.is_empty() {
        panic!("nave.obj vertex array empty — check model export");
    } else {
        eprintln!("nave.obj vertex count = {}", nave_vertex_array.len());
    }

    let planet_vertex_array = match Obj::load("./assets/sphere.obj") {
        Ok(sphere) => {
            eprintln!("Loaded ./assets/sphere.obj successfully");
            sphere.get_vertex_array()
        },
        Err(_) => {
            eprintln!("Warning: ./assets/sphere.obj not found — using nave mesh as fallback for planets");
            nave_vertex_array.clone()
        }
    };

    framebuffer.set_background_color(Color::new(25, 25, 75, 255));

    let sun = CelestialBody {
        name: "Sun".to_string(),
        translation: Vector3::new(0.0_f32, 0.0_f32, 0.0_f32),
        scale: 15.0_f32,
        rotation: Vector3::new(0.0_f32, 0.0_f32, 0.0_f32),
        orbit_radius: 0.0_f32,
        orbit_speed: 0.0_f32,
        rotation_speed: 0.5_f32,
        color: Color::new(255, 255, 0, 255),
    };
    let mercury = CelestialBody {
        name: "Mercury".to_string(),
        translation: Vector3::new(0.0_f32, 0.0_f32, 0.0_f32),
        scale: 2.0_f32,
        rotation: Vector3::new(0.0_f32, 0.0_f32, 0.0_f32),
        orbit_radius: 15.0_f32,
        orbit_speed: 0.8_f32,
        rotation_speed: 2.0_f32,
        color: Color::new(169, 169, 169, 255),
    };
    let earth = CelestialBody {
        name: "Earth".to_string(),
        translation: Vector3::new(0.0_f32, 0.0_f32, 0.0_f32),
        scale: 3.0_f32,
        rotation: Vector3::new(0.0_f32, 0.0_f32, 0.0_f32),
        orbit_radius: 25.0_f32,
        orbit_speed: 0.5_f32,
        rotation_speed: 1.5_f32,
        color: Color::new(0, 100, 200, 255),
    };
    let mars = CelestialBody {
        name: "Mars".to_string(),
        translation: Vector3::new(0.0_f32, 0.0_f32, 0.0_f32),
        scale: 2.5_f32,
        rotation: Vector3::new(0.0_f32, 0.0_f32, 0.0_f32),
        orbit_radius: 35.0_f32,
        orbit_speed: 0.3_f32,
        rotation_speed: 1.2_f32,
        color: Color::new(205, 92, 92, 255),
    };
    let uranus = CelestialBody {
        name: "Uranus".to_string(),
        translation: Vector3::new(0.0_f32, 0.0_f32, 0.0_f32),
        scale: 5.0_f32,
        rotation: Vector3::new(0.0_f32, 0.0_f32, 0.0_f32),
        orbit_radius: 45.0_f32,
        orbit_speed: 0.1_f32,
        rotation_speed: 0.8_f32,
        color: Color::new(173, 216, 230, 255),
    };

    let celestial_bodies = vec![sun, mercury.clone(), earth.clone(), mars.clone(), uranus.clone()];

    // 🌟 Definir posiciones de warp (animado)
    let warp_targets = [
        WarpTarget { eye: initial_camera_pos, target: initial_camera_target, up: initial_camera_up },
        WarpTarget {
            eye: Vector3::new(0.0_f32, 100.0_f32, 0.0_f32),
            target: Vector3::new(0.0_f32, 0.0_f32, 0.0_f32),
            up: Vector3::new(0.0_f32, 0.0_f32, -1.0_f32),
        },
        WarpTarget {
            eye: Vector3::new(0.0_f32, 20.0_f32, earth.orbit_radius + 20.0_f32),
            target: Vector3::new(0.0_f32, -15.0_f32, 0.0_f32),
            up: Vector3::new(0.0_f32, 1.0_f32, 0.0_f32),
        },
        WarpTarget {
            eye: Vector3::new(0.0_f32, 15.0_f32, mars.orbit_radius + 20.0_f32),
            target: Vector3::new(0.0_f32, -10.0_f32, 0.0_f32),
            up: Vector3::new(0.0_f32, 1.0_f32, 0.0_f32),
        },
        WarpTarget {
            eye: Vector3::new(0.0_f32, 10.0_f32, uranus.orbit_radius + 20.0_f32),
            target: Vector3::new(0.0_f32, -5.0_f32, 0.0_f32),
            up: Vector3::new(0.0_f32, 1.0_f32, 0.0_f32),
        },
    ];

    let mut time = 0.0_f32;
    let mut is_warping = false;
    let mut warp_start_time = 0.0_f32;
    let mut warp_duration = 1.0_f32; // segundos
    let mut current_warp_index = 0_usize;

    // Posición segura inicial de cámara (para restaurar si algo sale mal)
    let mut safe_camera_eye = camera.eye;
    let mut safe_camera_target = camera.target;

    // Parámetros para posicionar la nave relativa a la cámara (nave sigue la cámara)
    let nave_offset_back = 6.0_f32;        // cuánto queda detrás del ojo (positivo = atrás)
    let nave_offset_down = 2.5_f32;        // cuánto hacia abajo respecto al eye
    let default_nave_scale = 1.0_f32;      // ajustar según tu modelo
    let nave_model_offset_forward = 0.4_f32; // compensación por pivote del modelo (hacia el frente)

    // Parámetros de navegación libre (control 3D)
    let base_speed = 40.0_f32;      // unidades / s
    let sprint_mult = 2.2_f32;
    let yaw_speed = 1.8_f32;        // rad/s (flechas izquierda/derecha)
    let pitch_speed = 1.2_f32;      // rad/s (flechas arriba/abajo)

    while !window.window_should_close() {
        let dt = window.get_frame_time();
        time += dt;

        // Guardar posición segura previa
        let prev_eye = camera.eye;
        let prev_target = camera.target;

        // 🌟 Warping animado
        if !is_warping {
            for (i, key) in [
                KeyboardKey::KEY_ONE,
                KeyboardKey::KEY_TWO,
                KeyboardKey::KEY_THREE,
                KeyboardKey::KEY_FOUR,
                KeyboardKey::KEY_FIVE,
            ]
            .iter()
            .enumerate()
            {
                if window.is_key_pressed(*key) && i < warp_targets.len() {
                    is_warping = true;
                    warp_start_time = time;
                    current_warp_index = i;
                }
            }
        }

        if is_warping {
            let t = ((time - warp_start_time) / warp_duration).min(1.0_f32);
            let eased_t = ease_in_out(t);

            // en lugar de `camera.clone()` tomamos los campos directamente
            let start_eye = camera.eye;
            let start_target = camera.target;
            let start_up = camera.up;
            // Si `Camera` tiene yaw/pitch/distance expuestos los leemos directamente
            // (tu código original los usa, así que los copiamos aquí)
            let start_yaw = camera.yaw;
            let start_pitch = camera.pitch;
            let start_distance = camera.distance;

            let target_cam = warp_targets[current_warp_index].to_camera_state();

            // interpolamos campos
            camera.eye = lerp_vec3(start_eye, target_cam.eye, eased_t);
            camera.target = lerp_vec3(start_target, target_cam.target, eased_t);
            camera.up = lerp_vec3(start_up, target_cam.up, eased_t);

            camera.yaw = start_yaw + (target_cam.yaw - start_yaw) * eased_t;
            camera.pitch = start_pitch + (target_cam.pitch - start_pitch) * eased_t;
            camera.distance = start_distance + (target_cam.distance - start_distance) * eased_t;

            if t >= 1.0 {
                is_warping = false;
                // Asegurar valores exactos al final
                camera = warp_targets[current_warp_index].to_camera_state();
            }
        } else {
            // CONTROL 3D MANUAL: WASD = movimiento en el plano de la mirada, Q/E = down/up,
            // Shift = sprint, flechas = rotación yaw/pitch
            let mut speed = base_speed;
            if window.is_key_down(KeyboardKey::KEY_LEFT_SHIFT) {
                speed *= sprint_mult;
            }

            // Rotación con flechas
            if window.is_key_down(KeyboardKey::KEY_LEFT) {
                camera.yaw -= yaw_speed * dt;
            }
            if window.is_key_down(KeyboardKey::KEY_RIGHT) {
                camera.yaw += yaw_speed * dt;
            }
            if window.is_key_down(KeyboardKey::KEY_UP) {
                camera.pitch = clamp_f32(camera.pitch + pitch_speed * dt, -1.4_f32, 1.4_f32);
            }
            if window.is_key_down(KeyboardKey::KEY_DOWN) {
                camera.pitch = clamp_f32(camera.pitch - pitch_speed * dt, -1.4_f32, 1.4_f32);
            }

            // Dirección forward a partir de yaw/pitch
            let forward = Vector3::new(
                camera.yaw.cos() * camera.pitch.cos(),
                camera.pitch.sin(),
                camera.yaw.sin() * camera.pitch.cos(),
            );
            let forward_n = normalize_vec3(forward);
            let right_n = normalize_vec3(Vector3::new(forward_n.z, 0.0_f32, -forward_n.x));
            let up = Vector3::new(0.0_f32, 1.0_f32, 0.0_f32);

            // Movimiento local: W/S adelante/atrás, A/D strafe, Q baja, E sube
            if window.is_key_down(KeyboardKey::KEY_W) {
                camera.eye = add_vec3(camera.eye, mul_vec3_scalar(forward_n, speed * dt));
            }
            if window.is_key_down(KeyboardKey::KEY_S) {
                camera.eye = add_vec3(camera.eye, mul_vec3_scalar(forward_n, -speed * dt));
            }
            if window.is_key_down(KeyboardKey::KEY_A) {
                camera.eye = add_vec3(camera.eye, mul_vec3_scalar(right_n, -speed * dt));
            }
            if window.is_key_down(KeyboardKey::KEY_D) {
                camera.eye = add_vec3(camera.eye, mul_vec3_scalar(right_n, speed * dt));
            }
            if window.is_key_down(KeyboardKey::KEY_E) {
                camera.eye = add_vec3(camera.eye, mul_vec3_scalar(up, speed * dt));
            }
            if window.is_key_down(KeyboardKey::KEY_Q) {
                camera.eye = add_vec3(camera.eye, mul_vec3_scalar(up, -speed * dt));
            }

            // Actualizar target para que la cámara mire en la dirección definida por yaw/pitch
            camera.target = add_vec3(camera.eye, forward_n);
        }

        // Evitar colisiones y ajustar cámara (ya existente)
        let (adjusted_eye, adjusted_target) = avoid_collision(camera.eye, camera.target, &celestial_bodies, time);
        camera.eye = adjusted_eye;
        camera.target = adjusted_target;

        // Protección: si cámara contiene NaN/Inf o valores extremadamente grandes, restaurar a valor seguro
        let eye_ok = camera.eye.x.is_finite() && camera.eye.y.is_finite() && camera.eye.z.is_finite();
        let target_ok = camera.target.x.is_finite() && camera.target.y.is_finite() && camera.target.z.is_finite();
        let max_coord = 1e6_f32;
        let not_too_big = |v: &Vector3| v.x.abs() < max_coord && v.y.abs() < max_coord && v.z.abs() < max_coord;
        if !eye_ok || !target_ok || !not_too_big(&camera.eye) || !not_too_big(&camera.target) {
            // restaurar
            camera.eye = safe_camera_eye;
            camera.target = safe_camera_target;
        } else {
            // actualizar safe if everything is fine
            safe_camera_eye = camera.eye;
            safe_camera_target = camera.target;
        }

        framebuffer.clear();

        // 🌟 Renderizar skybox PRIMERO (más atrás)
        let view_matrix = camera.get_view_matrix();
        let projection_matrix = create_projection_matrix(PI / 3.0, window_width as f32 / window_height as f32, 0.1_f32, 1000.0_f32);
        let viewport_matrix = create_viewport_matrix(0.0_f32, 0.0_f32, window_width as f32, window_height as f32);
        render_skybox(&mut framebuffer, &view_matrix, &projection_matrix, &viewport_matrix, time);

        // Renderizar planetas
        // Renderizar planetas (se mantiene), pero añadir culling por distancia (evita renderar cuerpos demasiado próximos con triangulación muy densa)
        let max_render_distance = 5000.0_f32; // puedes ajustar
        for mut body in celestial_bodies.clone() {
            if body.name != "Sun" {
                body.translation.x = (time * body.orbit_speed).cos() * body.orbit_radius;
                body.translation.z = (time * body.orbit_speed).sin() * body.orbit_radius;
            }
            body.rotation.y += dt * body.rotation_speed;

            // distancia cámara <-> body
            let dx = camera.eye.x - body.translation.x;
            let dy = camera.eye.y - body.translation.y;
            let dz = camera.eye.z - body.translation.z;
            let dist_sq = dx*dx + dy*dy + dz*dz;
            if dist_sq > max_render_distance * max_render_distance {
                // omitimos objetos muy lejanos (mejora rendimiento)
                continue;
            }

            let model_matrix = create_model_matrix(body.translation, body.scale, body.rotation);
            let uniforms = Uniforms {
                model_matrix,
                view_matrix: camera.get_view_matrix(),
                projection_matrix,
                viewport_matrix,
                time,
                dt,
            };
            render(&mut framebuffer, &uniforms, &planet_vertex_array, &light, &body.name);
        }

        // Renderizar órbitas
        for body in &celestial_bodies {
            if body.name != "Sun" {
                let orbit_color = Color::new(255, 255, 255, 50);
                draw_orbit_3d(&mut framebuffer, body.orbit_radius, orbit_color, &view_matrix, &projection_matrix, &viewport_matrix);
            }
        }

        // La nave sigue a la cámara: calcular posición detrás y un poco abajo respecto a camera.eye (visible y acompañando)
        {
            let mut forward = sub_vec3(camera.target, camera.eye);
            forward = normalize_vec3(forward);
            let up = Vector3::new(0.0_f32, 1.0_f32, 0.0_f32);

            // colocar la nave ligeramente detrás y abajo del eye para que acompañe la cámara y sea visible
            let offset_back = mul_vec3_scalar(forward, -nave_offset_back);
            let offset_down = mul_vec3_scalar(up, -nave_offset_down);
            let offset_model = mul_vec3_scalar(forward, -nave_model_offset_forward);
            let nave_position = add_vec3(camera.eye, add_vec3(add_vec3(offset_back, offset_down), offset_model));

            let yaw = forward.z.atan2(forward.x);
            let fy = clamp_f32(forward.y, -1.0_f32, 1.0_f32);
            let pitch = fy.asin();

            let nave_model_matrix = create_model_matrix(
                nave_position,
                default_nave_scale,
                Vector3::new(pitch, yaw, 0.0_f32),
            );

            let uniforms = Uniforms {
                model_matrix: nave_model_matrix,
                view_matrix: camera.get_view_matrix(),
                projection_matrix,
                viewport_matrix,
                time,
                dt,
            };
            render(&mut framebuffer, &uniforms, &nave_vertex_array, &light, "Nave");
        }

        framebuffer.swap_buffers(&mut window, &raylib_thread);
        thread::sleep(Duration::from_millis(16));
    }
}
