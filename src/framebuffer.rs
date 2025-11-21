use raylib::prelude::*;

pub struct Framebuffer {
    pub width: i32,
    pub height: i32,
    pub color_buffer: Image,
    background_color: Color,
    current_color: Color,
    depth_buffer: Vec<f32>,
}

impl Framebuffer {
    pub fn new(width: i32, height: i32) -> Self {
        let background_color = Color::BLACK; // Un color por defecto
        let color_buffer = Image::gen_image_color(width, height, background_color);
        let depth_buffer = vec![f32::INFINITY; (width * height) as usize];
        Framebuffer {
            width,
            height,
            color_buffer,
            background_color,
            current_color: Color::WHITE,
            depth_buffer,
        }
    }

    pub fn clear(&mut self) {
        self.color_buffer.clear_background(self.background_color);
        self.depth_buffer.fill(f32::INFINITY);
    }
    
    pub fn point(&mut self, x: i32, y: i32, color: Vector3, depth: f32) {
        if x >= 0 && x < self.width && y >= 0 && y < self.height {
            let index = (y * self.width + x) as usize;

            if depth < self.depth_buffer[index] {
                self.depth_buffer[index] = depth;
                let pixel_color = Color::new(
                    (color.x.clamp(0.0, 1.0) * 255.0) as u8,
                    (color.y.clamp(0.0, 1.0) * 255.0) as u8,
                    (color.z.clamp(0.0, 1.0) * 255.0) as u8,
                    255,
                );
                self.color_buffer.draw_pixel(x, y, pixel_color);
            }
        }
    }
    
    // Método para dibujar una línea con profundidad específica
    pub fn draw_line_with_depth(&mut self, x0: i32, y0: i32, x1: i32, y1: i32, color: Color, depth: f32) {
        let mut x0 = x0;
        let mut y0 = y0;
        let x1 = x1;
        let y1 = y1;
        
        let dx = (x1 - x0).abs();
        let dy = (y1 - y0).abs();
        let sx = if x0 < x1 { 1 } else { -1 };
        let sy = if y0 < y1 { 1 } else { -1 };
        let mut err = dx - dy;
        
        loop {
            // Convertir el color de raylib a Vector3 para usar en point
            let color_vec3 = Vector3::new(
                color.r as f32 / 255.0,
                color.g as f32 / 255.0,
                color.b as f32 / 255.0
            );
            
            // Usar point con la profundidad especificada
            self.point(x0, y0, color_vec3, depth);
            
            if x0 == x1 && y0 == y1 {
                break;
            }
            
            let e2 = 2 * err;
            if e2 > -dy {
                err -= dy;
                x0 += sx;
            }
            if e2 < dx {
                err += dx;
                y0 += sy;
            }
        }
    }
    
    pub fn set_background_color(&mut self, color: Color) {
        self.background_color = color;
    }

    pub fn set_current_color(&mut self, color: Color) {
        self.current_color = color;
    }

    pub fn swap_buffers(&self, d: &mut RaylibHandle, thread: &RaylibThread) {
        if let Ok(texture) = d.load_texture_from_image(thread, &self.color_buffer) {
            let mut d = d.begin_drawing(thread);
            d.clear_background(self.background_color);
            d.draw_texture(&texture, 0, 0, Color::WHITE);
        }
    } 
}