// shaders.rs

use raylib::prelude::*;
use crate::vertex::Vertex;
use crate::Uniforms;
use crate::matrix::multiply_matrix_vector4;
use crate::fragment::Fragment;

// Helper para normalizar vector3
fn normalize_vec3(v: Vector3) -> Vector3 {
    let len = (v.x * v.x + v.y * v.y + v.z * v.z).sqrt();
    if len > 0.0 { 
        Vector3::new(v.x / len, v.y / len, v.z / len) 
    } else { 
        v 
    }
}

// Helper para latitud
fn lat_factor(lat: f32) -> f32 {
    (lat - 0.5).abs() * 2.0
}

// Función de ruido solar
fn solar_noise(x: f32, y: f32, z: f32, time: f32) -> f32 {
    let n1 = (x * 3.0 + time * 0.7).sin() * (y * 2.0 + time * 0.5).cos() * (z * 4.0 + time * 0.3).sin();
    let n2 = (x * 6.0 + time * 1.2).cos() * (y * 3.0 + time * 0.8).sin() * (z * 2.0 + time * 1.1).cos();
    let n3 = (x * 12.0 + time * 2.0).sin() * (y * 8.0 + time * 1.5).cos() * (z * 6.0 + time * 0.9).sin();
    (n1 * 0.5 + n2 * 0.3 + n3 * 0.2).abs()
}

pub fn vertex_shader(vertex: &Vertex, uniforms: &Uniforms) -> Vertex {
    let position_vec4 = Vector4::new(vertex.position.x, vertex.position.y, vertex.position.z, 1.0);
    let world_position = multiply_matrix_vector4(&uniforms.model_matrix, &position_vec4);
    let view_position = multiply_matrix_vector4(&uniforms.view_matrix, &world_position);
    let clip_position = multiply_matrix_vector4(&uniforms.projection_matrix, &view_position);
    let ndc = if clip_position.w != 0.0 {
        Vector3::new(clip_position.x / clip_position.w, clip_position.y / clip_position.w, clip_position.z / clip_position.w)
    } else {
        Vector3::new(clip_position.x, clip_position.y, clip_position.z)
    };
    let ndc_vec4 = Vector4::new(ndc.x, ndc.y, ndc.z, 1.0);
    let screen_position = multiply_matrix_vector4(&uniforms.viewport_matrix, &ndc_vec4);
    Vertex {
        position: vertex.position,
        normal: vertex.normal,
        tex_coords: vertex.tex_coords,
        color: vertex.color,
        transformed_position: Vector3::new(screen_position.x, screen_position.y, screen_position.z),
        transformed_normal: vertex.normal,
    }
}

pub fn fragment_shader(fragment: &Fragment, _uniforms: &Uniforms) -> Vector3 {
    fragment.color
}

// 🌞 Sol mejorado
pub fn sun_fragment_shader(fragment: &Fragment, uniforms: &Uniforms) -> Vector3 {
    let pos = fragment.world_position;
    let time = uniforms.time;

    let turbulence = solar_noise(pos.x, pos.y, pos.z, time) * 0.8 +
                    solar_noise(pos.x * 2.3, pos.y * 2.1, pos.z * 1.8, time + 100.0) * 0.4 +
                    solar_noise(pos.x * 4.7, pos.y * 3.9, pos.z * 5.2, time + 200.0) * 0.2;
    
    let pulsation = 1.0 + (time * 0.5).sin() * 0.1;
    let distance_from_center = pos.length().min(1.0);
    let radial_attenuation = (1.0 - distance_from_center.powf(3.0)).max(0.0);

    let core_color = Vector3::new(1.0, 0.2, 0.0);
    let mid_color = Vector3::new(1.0, 0.6, 0.1);
    let outer_color = Vector3::new(1.0, 0.9, 0.4);
    let corona_color = Vector3::new(0.9, 0.95, 1.0);

    let base_color = if distance_from_center < 0.6 {
        core_color
    } else if distance_from_center < 0.85 {
        let t = (distance_from_center - 0.6) / 0.25;
        core_color * (1.0 - t) + mid_color * t
    } else if distance_from_center < 0.95 {
        let t = (distance_from_center - 0.85) / 0.1;
        mid_color * (1.0 - t) + outer_color * t
    } else {
        let t = (distance_from_center - 0.95) / 0.05;
        outer_color * (1.0 - t) + corona_color * t * 0.5
    };

    let intensity_mod = 1.0 + turbulence * 2.5;
    let flare_effect = (solar_noise(pos.x * 0.7, pos.y * 0.6, pos.z * 0.8, time * 1.5) * 1.2 + 0.2).min(1.2);
    let center_glow = (1.0 - distance_from_center).powf(8.0) * 2.0;

    let mut color = base_color * intensity_mod * pulsation * radial_attenuation;
    color = color * (1.0 + flare_effect * 0.5) + Vector3::new(1.0, 1.0, 0.8) * flare_effect * 0.6;
    color = color + Vector3::new(1.0, 0.9, 0.6) * center_glow;

    Vector3::new(
        color.x.min(2.0),
        color.y.min(1.8),
        color.z.min(1.5),
    )
}

// 🪐 Mercurio (agregado ahora — más realista que gris plano)
pub fn mercury_fragment_shader(fragment: &Fragment, uniforms: &Uniforms) -> Vector3 {
    let pos = fragment.world_position;
    let time = uniforms.time;

    // Patrón de cráteres y rocas
    let crater_noise = 
        ((pos.x * 10.0 + pos.z * 8.0).sin() * (pos.y * 7.0).cos() * 0.6).abs().powf(1.8);
    let terrain_noise = 
        ((pos.x * 5.0).sin() * (pos.z * 4.0).cos() * 0.5 + 0.5).abs();

    let dark_rock = Vector3::new(0.25, 0.23, 0.24);
    let light_rock = Vector3::new(0.55, 0.52, 0.50);
    let crater_deep = Vector3::new(0.15, 0.14, 0.15);

    let crater_factor = crater_noise.min(1.0);
    let terrain_factor = terrain_noise.min(1.0);

    let base_surface = dark_rock * (1.0 - terrain_factor) + light_rock * terrain_factor;
    let cratered_surface = base_surface * (1.0 - crater_factor * 0.5) + crater_deep * crater_factor * 0.5;

    // Iluminación simple
    let light_dir = normalize_vec3(Vector3::new(1.0, 1.0, 1.0));
    let dot = pos.dot(light_dir).max(0.0); // ✅ sin & aquí
    let lit_color = cratered_surface * dot.max(0.3);

    Vector3::new(lit_color.x.min(1.0), lit_color.y.min(1.0), lit_color.z.min(1.0))
}

// 🌍 Tierra
pub fn earth_fragment_shader(fragment: &Fragment, uniforms: &Uniforms) -> Vector3 {
    let pos = fragment.world_position;
    let time = uniforms.time;

    let longitude = (pos.z.atan2(pos.x) + std::f32::consts::PI) / (2.0 * std::f32::consts::PI);
    let latitude = (pos.y.asin() + std::f32::consts::PI / 2.0) / std::f32::consts::PI;

    let land_noise = 
        ((longitude * 6.0 + latitude * 2.0).sin() * 0.5 +
         (longitude * 3.0 + time * 0.05).cos() * 0.3 +
         (latitude * 8.0).sin() * 0.2).abs() * 2.0 - 0.7;

    let is_land = land_noise.max(0.0).min(1.0);

    let cloud_noise = 
        ((pos.x * 4.0 + time * 0.2).cos() * 0.4 +
         (pos.y * 5.0).sin() * 0.3 +
         (pos.z * 3.0 + time * 0.15).sin() * 0.3).abs() * 0.6 + 0.2;
    let cloud_factor = cloud_noise.min(1.0);

    let ocean_color = Vector3::new(0.05, 0.15, 0.5);
    let shallow_ocean = Vector3::new(0.2, 0.4, 0.8);
    let land_base = Vector3::new(0.35, 0.5, 0.2);
    let desert = Vector3::new(0.7, 0.6, 0.3);
    let ice = Vector3::new(0.85, 0.9, 0.95);

    let land_color = if latitude > 0.7 || latitude < 0.3 {
        ice * (latitude - 0.7).abs().max(0.0).min(0.3) * 3.3 + land_base * (1.0 - (latitude - 0.7).abs().max(0.0).min(0.3) * 3.3)
    } else if latitude > 0.55 || latitude < 0.45 {
        desert * (latitude - 0.55).abs().max(0.0).min(0.1) * 10.0 + land_base * (1.0 - (latitude - 0.55).abs().max(0.0).min(0.1) * 10.0)
    } else {
        land_base
    };

    let surface_color = ocean_color * (1.0 - is_land) + land_color * is_land;
    let coast_blend = (0.2 - (is_land - 0.1).abs()).max(0.0) * 5.0;
    let blended_surface = surface_color * (1.0 - coast_blend) + shallow_ocean * coast_blend;

    let cloud_color = Vector3::new(0.95, 0.97, 1.0);
    let final_color = blended_surface * (1.0 - cloud_factor * 0.6) + cloud_color * cloud_factor * 0.6;

    // ✅ Corregido: sin &
    let light_dir = normalize_vec3(Vector3::new(1.0, 1.0, 1.0));
    let dot = pos.dot(light_dir).max(0.0); // ✅ aquí estaba el error
    let lit_color = final_color * dot.max(0.2);

    Vector3::new(lit_color.x.min(1.0), lit_color.y.min(1.0), lit_color.z.min(1.0))
}

// 🔴 Marte
pub fn mars_fragment_shader(fragment: &Fragment, uniforms: &Uniforms) -> Vector3 {
    let pos = fragment.world_position;
    let time = uniforms.time;

    let longitude = (pos.z.atan2(pos.x) + std::f32::consts::PI) / (2.0 * std::f32::consts::PI);
    let latitude = (pos.y.asin() + std::f32::consts::PI / 2.0) / std::f32::consts::PI;

    let terrain_base = 
        ((longitude * 10.0 + latitude * 3.0).sin() * 0.4 +
         (longitude * 5.0 + time * 0.02).cos() * 0.3 +
         (latitude * 7.0).sin() * 0.3).abs() * 1.2 - 0.5;

    let crater_noise = 
        ((pos.x * 15.0).sin() * (pos.y * 12.0).cos() * (pos.z * 10.0).sin() * 0.6).abs().powf(1.5);

    let dust_factor = (0.5 - (latitude - 0.5).abs()).max(0.0) * 0.8 + 0.2;
    let dust_noise = ((pos.x * 20.0 + time * 0.3).cos() * 0.7 + 0.3).max(0.0);
    let dust = dust_factor * dust_noise;

    let base_mars = Vector3::new(0.85, 0.45, 0.25);
    let dark_rock = Vector3::new(0.5, 0.25, 0.15);
    let light_dust = Vector3::new(0.95, 0.7, 0.45);
    let ice_caps = Vector3::new(0.85, 0.9, 0.95);

    let terrain_factor = (terrain_base * 0.6 + 0.4).max(0.0).min(1.0);
    let crater_factor = crater_noise.min(1.0);

    let rocky_color = base_mars * (1.0 - terrain_factor) + dark_rock * terrain_factor;
    let cratered_color = rocky_color * (1.0 - crater_factor * 0.5) + dark_rock * crater_factor * 0.5;

    let polar_blend = (lat_factor(latitude) - 0.8).max(0.0) * 5.0;
    let final_surface = cratered_color * (1.0 - polar_blend) + ice_caps * polar_blend;

    let dusty_color = final_surface * (1.0 - dust * 0.3) + light_dust * dust * 0.3;

    // ✅ Corregido: sin &
    let light_dir = normalize_vec3(Vector3::new(1.0, 1.0, 1.0));
    let dot = pos.dot(light_dir).max(0.0); // ✅ aquí estaba el error
    let lit_color = dusty_color * dot.max(0.2);

    Vector3::new(lit_color.x.min(1.0), lit_color.y.min(1.0), lit_color.z.min(1.0))
}

// 🪐 Urano
pub fn uranus_fragment_shader(fragment: &Fragment, uniforms: &Uniforms) -> Vector3 {
    let pos = fragment.world_position;
    let time = uniforms.time;

    let latitude = (pos.y.asin() / (std::f32::consts::PI / 2.0)).abs();

    let band_noise = ((latitude * 10.0 + time * 0.1).sin() * 0.4 + 0.6).max(0.0).min(1.0);
    let small_clouds = ((pos.x * 12.0 + time * 0.3).cos() * (pos.z * 8.0).sin() * 0.5 + 0.5).max(0.0).min(1.0);

    let base = Vector3::new(0.55, 0.80, 0.88);
    let band_dark = Vector3::new(0.45, 0.70, 0.80);
    let band_light = Vector3::new(0.65, 0.85, 0.92);
    let high_clouds = Vector3::new(0.90, 0.95, 1.0);

    let banded_color = base * (1.0 - band_noise * 0.2) + 
                      (band_dark * 0.5 + band_light * 0.5) * band_noise * 0.2;

    let final_color = banded_color * (1.0 - small_clouds * 0.25) + high_clouds * small_clouds * 0.25;

    let polar_glow = (1.0 - latitude).powf(4.0) * 0.3;
    let glow_color = Vector3::new(0.7, 0.9, 1.0) * polar_glow;

    // ✅ Corregido: sin &
    let light_dir = normalize_vec3(Vector3::new(1.0, 1.0, 1.0));
    let dot = pos.dot(light_dir).max(0.0); // ✅ aquí estaba el error
    let lit_color = (final_color + glow_color) * dot.max(0.3);

    Vector3::new(lit_color.x.min(1.0), lit_color.y.min(1.0), lit_color.z.min(1.0))
}

// 🚀 Nave
pub fn nave_fragment_shader(fragment: &Fragment, uniforms: &Uniforms) -> Vector3 {
    let pos = fragment.world_position;
    let time = uniforms.time;
    let metal_pattern = (pos.x * 10.0).sin() * (pos.y * 8.0).cos() * (pos.z * 6.0).sin();
    let panel_pattern = (pos.x * 15.0 + pos.z * 10.0).sin() * (pos.y * 12.0).cos();
    let base_color = Vector3::new(0.6, 0.6, 0.7);
    let panel_color = Vector3::new(0.4, 0.45, 0.5);
    let accent_color = Vector3::new(0.8, 0.8, 0.9);
    let pattern_factor = (metal_pattern * 0.3 + 0.7).max(0.0).min(1.0);
    let panel_factor = (panel_pattern * 0.2 + 0.8).max(0.0).min(1.0);
    let textured_surface = base_color * (1.0 - pattern_factor) + panel_color * pattern_factor;
    let final_color = textured_surface * (1.0 - panel_factor * 0.2) + accent_color * panel_factor * 0.2;
    let lighting = (pos.y * 0.4 + 0.6).max(0.3);
    let lit_color = final_color * lighting;
    let light_pulse = (time * 2.0).sin().abs() * 0.1 + 0.9;
    let pulsed_color = Vector3::new(0.9, 0.95, 1.0) * light_pulse * 0.1 + lit_color * (1.0 - 0.1);
    Vector3::new(pulsed_color.x.clamp(0.0, 1.0), pulsed_color.y.clamp(0.0, 1.0), pulsed_color.z.clamp(0.0, 1.0))
}

// 🌟 Skybox
pub fn skybox_fragment_shader(fragment: &Fragment, _uniforms: &Uniforms) -> Vector3 {
    Vector3::new(1.0, 1.0, 1.0)
}