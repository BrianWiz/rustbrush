use crate::{Brush, ALPHA_CHANNEL, BLUE_CHANNEL, GREEN_CHANNEL, RED_CHANNEL};



/// Paints a brush stroke on the pixel buffer
/// Example usage:
/// ```rust
/// PaintOperation {
///     pixel_buffer: &mut self.layers[self.current_layer],
///     brush: &Brush::default(),
///     color: [255, 0, 0], // Red
///     buffer_width: self.buffer_width,
///     buffer_height: self.buffer_height,
///     cursor_position,
///     last_cursor_position,
///     is_eraser: false,
/// }
///     .process();
/// ```
pub struct PaintOperation<'a> {
    /// Pixel buffer to paint on. This should be a 1D (flat) array of RGBA values
    /// Example (4 pixels):
    /// ```rust
    /// [r, g, b, a, r, g, b, a, r, g, b, a, r, g, b, a]
    /// ```
    pub pixel_buffer: &'a mut Vec<u8>,
    pub pixel_buffer_width: u32,
    pub pixel_buffer_height: u32,
    pub brush: &'a Brush,
    pub color: [u8; 3],
    pub cursor_position: (f32, f32),
    pub last_cursor_position: (f32, f32),
    pub is_eraser: bool,
}

impl PaintOperation<'_> {
    pub fn process(self) {
        let (x0, y0) = (self.last_cursor_position.0, self.last_cursor_position.1);
        let (x1, y1) = (self.cursor_position.0, self.cursor_position.1);
        
        let dx = x1 - x0;
        let dy = y1 - y0;
        let distance = ((dx * dx + dy * dy) as f32).sqrt();
        
        let min_spacing = self.brush.radius() * self.brush.spacing();
        let steps = (distance / min_spacing).max(1.0) as i32;
        
        let stamp = self.brush.compute_stamp(self.color);
        
        for i in 0..=steps {
            let t = i as f32 / steps as f32;
            let x = x0 + dx * t;
            let y = y0 + dy * t;
            
            for pixel in &stamp.pixels {
                let px = (x + pixel.x as f32) as i32;
                let py = (y + pixel.y as f32) as i32;
                
                if target_px_in_bounds((px, py), self.pixel_buffer_width, self.pixel_buffer_height) {
                    let index = (py * self.pixel_buffer_width as i32 + px) as usize * 4;

                    let alpha = pixel.color[ALPHA_CHANNEL] as f32 / 255.0;
                    if self.is_eraser {
                        let current_alpha = self.pixel_buffer[index + 3] as f32 / 255.0;
                        let erase_strength = alpha * self.brush.opacity();
                        self.pixel_buffer[index + ALPHA_CHANNEL] = ((current_alpha * (1.0 - erase_strength)) * 255.0) as u8;
                    } else {
                        for c in 0..4 {
                            let current = self.pixel_buffer[index + c] as f32 / 255.0;
                            let new = pixel.color[c] as f32 / 255.0;
                            let result = current + (new * (1.0 - current));
                            self.pixel_buffer[index + c] = (result * 255.0) as u8;
                        }
                    }
                }
            }
        }
    }
}

/// Smears the pixel buffer
pub struct SmearOperation<'a> {
    pub pixel_buffer: &'a mut Vec<u8>,
    pub pixel_buffer_width: u32,
    pub pixel_buffer_height: u32,
    pub brush: &'a Brush,
    pub cursor_position: (f32, f32),
    pub last_cursor_position: (f32, f32),
    pub smear_strength: f32,
}

impl SmearOperation<'_> {
    pub fn process(self) {
        let (x0, y0) = (self.last_cursor_position.0, self.last_cursor_position.1);
        let (x1, y1) = (self.cursor_position.0, self.cursor_position.1);
    
        let dx = x1 - x0;
        let dy = y1 - y0;
        let distance = ((dx * dx + dy * dy) as f32).sqrt();
    
        let min_spacing = self.brush.radius() * self.brush.spacing();
        let steps = (distance / min_spacing).max(1.0) as i32;
    
        for i in 0..=steps {
            let t = i as f32 / steps as f32;
            let x = x0 + dx * t;
            let y = y0 + dy * t;
    
            for pixel in self.brush.compute_stamp([255, 255, 255]).pixels {
                let px = (x + pixel.x as f32) as i32;
                let py = (y + pixel.y as f32) as i32;
    
                if px >= 0 && px < self.pixel_buffer_width as i32 && py >= 0 && py < self.pixel_buffer_height as i32 {
                    let index = (py * self.pixel_buffer_width as i32 + px) as usize * 4;
    
                    let current_color = [
                        self.pixel_buffer[index + RED_CHANNEL],
                        self.pixel_buffer[index + GREEN_CHANNEL],
                        self.pixel_buffer[index + BLUE_CHANNEL],
                    ];
    
                    let current_alpha = if self.pixel_buffer.len() > index + 3 {
                        self.pixel_buffer[index + ALPHA_CHANNEL] as f32 / 255.0
                    } else {
                        1.0
                    };
    
                    let smear_dx = -dx * self.smear_strength;
                    let smear_dy = -dy * self.smear_strength;
    
                    let target_px = (px as f32 + smear_dx) as i32;
                    let target_py = (py as f32 + smear_dy) as i32;
    
                    if target_px_in_bounds((target_px, target_py), self.pixel_buffer_width, self.pixel_buffer_height) {
                        let target_index = (target_py * self.pixel_buffer_width as i32 + target_px) as usize * 4;
    
                        let target_alpha = self.pixel_buffer[target_index + ALPHA_CHANNEL] as f32 / 255.0;
                        let stamp_alpha = pixel.color[ALPHA_CHANNEL] as f32 / 255.0;

                        let smear_strength = stamp_alpha * self.smear_strength;
    
                        if smear_strength > 0.0 {
                            for c in 0..3 {
                                let current = current_color[c] as f32 / 255.0;
                                let target = self.pixel_buffer[target_index + c] as f32 / 255.0;
                                let result = current + (target - current) * smear_strength;
                                self.pixel_buffer[index + c] = (result * 255.0) as u8;
                            }
    
                            let result = current_alpha + (target_alpha - current_alpha) * smear_strength;
                            self.pixel_buffer[index + ALPHA_CHANNEL] = (result * 255.0) as u8;
                        }
                    }
                }
            }
        }
    }
}

fn target_px_in_bounds(target_px: (i32, i32), buffer_width: u32, buffer_height: u32) -> bool {
    target_px.0 >= 0 && target_px.0 < buffer_width as i32 && target_px.1 >= 0 && target_px.1 < buffer_height as i32
}