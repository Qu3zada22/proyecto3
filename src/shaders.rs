// shaders.rs
use raylib::prelude::*;
use crate::vertex::Vertex;
use crate::Uniforms;
use crate::matrix::multiply_matrix_vector4;
use crate::fragment::Fragment;

fn transform_normal(normal: &Vector3, model_matrix: &Matrix) -> Vector3 {
    let normal_vec4 = Vector4::new(normal.x, normal.y, normal.z, 0.0);
    let transformed_normal_vec4 = multiply_matrix_vector4(model_matrix, &normal_vec4);
    let mut transformed_normal = Vector3::new(
        transformed_normal_vec4.x,
        transformed_normal_vec4.y,
        transformed_normal_vec4.z,
    );
    transformed_normal.normalize();
    transformed_normal
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
        transformed_normal: transform_normal(&vertex.normal, &uniforms.model_matrix),
    }
}

fn solar_noise(x: f32, y: f32, z: f32, time: f32) -> f32 {
    let n1 = (x * 3.0 + time * 0.7).sin() * (y * 2.0 + time * 0.5).cos() * (z * 4.0 + time * 0.3).sin();
    let n2 = (x * 6.0 + time * 1.2).cos() * (y * 3.0 + time * 0.8).sin() * (z * 2.0 + time * 1.1).cos();
    let n3 = (x * 12.0 + time * 2.0).sin() * (y * 8.0 + time * 1.5).cos() * (z * 6.0 + time * 0.9).sin();
    (n1 * 0.5 + n2 * 0.3 + n3 * 0.2).abs()
}

pub fn fragment_shader(fragment: &Fragment, _uniforms: &Uniforms) -> Vector3 {
    fragment.color
}

pub fn sun_fragment_shader(fragment: &Fragment, uniforms: &Uniforms) -> Vector3 {
    let pos = fragment.world_position;
    let time = uniforms.time;
    let turbulence = solar_noise(pos.x, pos.y, pos.z, time) * 0.6 +
                    solar_noise(pos.x * 2.0, pos.y * 2.0, pos.z * 2.0, time + 100.0) * 0.3 +
                    solar_noise(pos.x * 4.0, pos.y * 4.0, pos.z * 4.0, time + 200.0) * 0.1;
    let pulsation = (time * 1.0).sin().abs() * 0.2 + 0.8;
    let distance_from_center = pos.length();
    let zone_factor = if distance_from_center < 0.7 {
        0.0
    } else if distance_from_center < 0.9 {
        (distance_from_center - 0.7) / 0.2
    } else {
        (distance_from_center - 0.9) / 0.1
    }.min(1.0);
    let core_color = Vector3::new(1.0, 0.3, 0.1);
    let surface_color = Vector3::new(1.0, 0.6, 0.2);
    let corona_color = Vector3::new(1.0, 0.9, 0.4);
    let base_color = if zone_factor < 0.5 {
        let t = zone_factor * 2.0;
        core_color * (1.0 - t) + surface_color * t
    } else {
        let t = (zone_factor - 0.5) * 2.0;
        surface_color * (1.0 - t) + corona_color * t
    };
    let intensity = (turbulence * 1.5 + pulsation) * 0.8;
    let solar_flare_noise = solar_noise(pos.x * 0.5, pos.y * 0.5, pos.z * 0.5, time * 2.0);
    let flare_effect = (solar_flare_noise * 2.0 + (time * 3.0).sin().abs() * 0.5).min(1.0);
    let final_color = base_color * intensity * (1.0 - flare_effect * 0.3) + 
                     Vector3::new(1.0, 1.0, 0.8) * flare_effect * 0.7;
    Vector3::new(final_color.x.clamp(0.0, 1.0), final_color.y.clamp(0.0, 1.0), final_color.z.clamp(0.0, 1.0))
}

pub fn mercury_fragment_shader(fragment: &Fragment, uniforms: &Uniforms) -> Vector3 {
    let pos = fragment.world_position;
    let crater_pattern = (pos.x * 8.0).sin() * (pos.y * 8.0).cos() * (pos.z * 8.0).sin();
    let crater_depth = (crater_pattern * 0.5 + 0.5).powf(3.0);
    let surface_noise = (pos.x * 15.0 + pos.z * 10.0).sin() * (pos.y * 12.0).cos();
    let rocky_pattern = (surface_noise * 0.3 + 0.7).abs();
    let dark_surface = Vector3::new(0.3, 0.3, 0.35);
    let light_surface = Vector3::new(0.55, 0.5, 0.48);
    let crater_color = Vector3::new(0.2, 0.2, 0.22);
    let base_color = if crater_depth < 0.3 {
        crater_color * (1.0 - crater_depth * 2.0) + dark_surface * crater_depth * 2.0
    } else {
        dark_surface * (1.0 - crater_depth) + light_surface * crater_depth
    };
    let textured_color = base_color * rocky_pattern;
    let sun_exposure = (pos.y + 1.0) * 0.5;
    let sun_reflection = Vector3::new(0.7, 0.65, 0.6) * sun_exposure * 0.15;
    let final_color = textured_color + sun_reflection;
    Vector3::new(final_color.x.clamp(0.0, 1.0), final_color.y.clamp(0.0, 1.0), final_color.z.clamp(0.0, 1.0))
}

pub fn earth_fragment_shader(fragment: &Fragment, uniforms: &Uniforms) -> Vector3 {
    let pos = fragment.world_position;
    let time = uniforms.time;
    let land_pattern = (pos.x * 5.0 + time * 0.2).sin() * (pos.z * 3.0).cos();
    let cloud_pattern = (pos.x * 8.0 + time * 0.3).cos() * (pos.y * 6.0).sin();
    let ocean_color = Vector3::new(0.1, 0.3, 0.7);
    let land_color = Vector3::new(0.2, 0.6, 0.2);
    let cloud_color = Vector3::new(0.9, 0.95, 1.0);
    let is_land = (land_pattern * 0.7 + 0.3).max(0.0).min(1.0);
    let is_cloud = (cloud_pattern * 0.4 + 0.6).max(0.0).min(1.0);
    let base_color = ocean_color * (1.0 - is_land) + land_color * is_land;
    let final_color = base_color * (1.0 - is_cloud * 0.4) + cloud_color * is_cloud * 0.4;
    let lighting = (pos.y * 0.5 + 0.5).max(0.3);
    let lit_color = final_color * lighting;
    Vector3::new(lit_color.x.clamp(0.0, 1.0), lit_color.y.clamp(0.0, 1.0), lit_color.z.clamp(0.0, 1.0))
}

pub fn mars_fragment_shader(fragment: &Fragment, uniforms: &Uniforms) -> Vector3 {
    let pos = fragment.world_position;
    let time = uniforms.time;
    let terrain_pattern = (pos.x * 6.0 + time * 0.1).sin() * (pos.z * 4.0).cos();
    let dust_pattern = (pos.x * 12.0 + pos.y * 8.0).sin() * (pos.z * 10.0).cos();
    let base_color = Vector3::new(0.8, 0.4, 0.2);
    let dark_rock = Vector3::new(0.6, 0.3, 0.15);
    let light_dust = Vector3::new(0.9, 0.6, 0.3);
    let terrain_factor = (terrain_pattern * 0.5 + 0.5).max(0.0).min(1.0);
    let dust_factor = (dust_pattern * 0.3 + 0.7).max(0.0).min(1.0);
    let rocky_surface = base_color * (1.0 - terrain_factor) + dark_rock * terrain_factor;
    let dusty_surface = rocky_surface * (1.0 - dust_factor) + light_dust * dust_factor;
    let lighting = (pos.y * 0.5 + 0.5).max(0.2);
    let lit_color = dusty_surface * lighting;
    Vector3::new(lit_color.x.clamp(0.0, 1.0), lit_color.y.clamp(0.0, 1.0), lit_color.z.clamp(0.0, 1.0))
}

pub fn uranus_fragment_shader(fragment: &Fragment, uniforms: &Uniforms) -> Vector3 {
    let pos = fragment.world_position;
    let time = uniforms.time;
    let band_pattern = (pos.y * 8.0 + time * 0.1).sin();
    let cloud_pattern = (pos.x * 6.0 + pos.z * 4.0 + time * 0.2).cos();
    let base_color = Vector3::new(0.6, 0.8, 0.9);
    let band_color = Vector3::new(0.5, 0.7, 0.85);
    let cloud_color = Vector3::new(0.7, 0.85, 0.95);
    let band_factor = (band_pattern * 0.4 + 0.6).max(0.0).min(1.0);
    let cloud_factor = (cloud_pattern * 0.3 + 0.7).max(0.0).min(1.0);
    let banded_surface = base_color * (1.0 - band_factor) + band_color * band_factor;
    let final_surface = banded_surface * (1.0 - cloud_factor * 0.3) + cloud_color * cloud_factor * 0.3;
    let lighting = (pos.y * 0.3 + 0.7).max(0.4);
    let lit_color = final_surface * lighting;
    Vector3::new(lit_color.x.clamp(0.0, 1.0), lit_color.y.clamp(0.0, 1.0), lit_color.z.clamp(0.0, 1.0))
}

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

// 🌟 Skybox shader (estrellas)
pub fn skybox_fragment_shader(fragment: &Fragment, _uniforms: &Uniforms) -> Vector3 {
    Vector3::new(1.0, 1.0, 1.0)
}