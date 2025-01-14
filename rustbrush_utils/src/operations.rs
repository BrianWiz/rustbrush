use ecolor::{Color32, Rgba};

use crate::{Brush, RgbaExtensions};

pub struct PaintOperation<'a> {
    pub pixel_buffer: &'a mut Vec<Color32>,
    pub canvas_width: u32,
    pub canvas_height: u32,
    pub brush: &'a Brush,
    pub color: Rgba,
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
        let distance = (dx * dx + dy * dy).sqrt();

        let min_spacing = self.brush.radius() * self.brush.spacing();
        let steps = (distance / min_spacing).max(1.0) as i32;

        let stamp = self.brush.compute_stamp();

        for i in 0..=steps {
            let t = i as f32 / steps as f32;
            let x = x0 + dx * t;
            let y = y0 + dy * t;

            for stamp_pixel in &stamp.pixels {
                let px = (x + stamp_pixel.x as f32) as i32;
                let py = (y + stamp_pixel.y as f32) as i32;

                if stamp_pixel.color.a() <= 0.0 {
                    continue;
                }

                if target_px_in_bounds((px, py), self.canvas_width, self.canvas_height) {
                    let index = (py * self.canvas_width as i32 + px) as usize;
                    let current_color = Rgba::from(self.pixel_buffer[index]);
                    let brush_color = Rgba::from_rgba_premultiplied(
                        self.color.r(),
                        self.color.g(),
                        self.color.b(),
                        self.color.a() * stamp_pixel.color.a(),
                    );

                    let final_color = Color32::from(brush_color.overlay(&current_color));
                    if final_color.a() > 0 {
                        self.pixel_buffer[index] = final_color;
                    }
                }
            }
        }
    }
}


pub struct SmudgeOperation<'a> {
    pub pixel_buffer: &'a mut Vec<Color32>,
    pub pixel_buffer_width: u32,
    pub pixel_buffer_height: u32,
    pub brush: &'a Brush,
    pub cursor_position: (f32, f32),
    pub last_cursor_position: (f32, f32),
    pub smudge_strength: f32,
}

impl SmudgeOperation<'_> {
    pub fn process(self) {
        let (x0, y0) = (self.last_cursor_position.0, self.last_cursor_position.1);
        let (x1, y1) = (self.cursor_position.0, self.cursor_position.1);

        let dx = x1 - x0;
        let dy = y1 - y0;
        let distance = (dx * dx + dy * dy).sqrt();

        let min_spacing = self.brush.radius() * self.brush.spacing();
        let steps = (distance / min_spacing).max(1.0) as i32;

        let stamp = self.brush.compute_stamp();

        for i in 0..=steps {
            let t = i as f32 / steps as f32;
            let x = x0 + dx * t;
            let y = y0 + dy * t;

            for stamp_pixel in &stamp.pixels {
                let px = (x + stamp_pixel.x as f32) as i32;
                let py = (y + stamp_pixel.y as f32) as i32;

                if target_px_in_bounds((px, py), self.pixel_buffer_width, self.pixel_buffer_height) {
                    let smudge_dx = -dx * self.smudge_strength;
                    let smudge_dy = -dy * self.smudge_strength;

                    let target_px = (px as f32 + smudge_dx) as i32;
                    let target_py = (py as f32 + smudge_dy) as i32;

                    if target_px_in_bounds(
                        (target_px, target_py),
                        self.pixel_buffer_width,
                        self.pixel_buffer_height,
                    ) {
                        let stamp_alpha = stamp_pixel.color.a();
                        let blend_strength = stamp_alpha * self.smudge_strength;

                        if blend_strength > 0.0 {
                            let index = (py * self.pixel_buffer_width as i32 + px) as usize;
                            let target_index =
                                (target_py * self.pixel_buffer_width as i32 + target_px) as usize;
                            let current_color = self.pixel_buffer[index];
                            let target_color = self.pixel_buffer[target_index];

                            let blend = |c1: u8, c2: u8, t: f32| -> u8 {
                                ((c1 as f32) * (1.0 - t) + (c2 as f32) * t) as u8
                            };

                            let src_alpha = target_color.a() as f32 / 255.0;
                            let dst_alpha = current_color.a() as f32 / 255.0;
                            let mix_factor = blend_strength;

                            let out_alpha = src_alpha * mix_factor + dst_alpha * (1.0 - mix_factor);

                            let new_color = Color32::from_rgba_premultiplied(
                                blend(current_color.r(), target_color.r(), mix_factor),
                                blend(current_color.g(), target_color.g(), mix_factor),
                                blend(current_color.b(), target_color.b(), mix_factor),
                                (out_alpha * 255.0) as u8,
                            );

                            self.pixel_buffer[index] = new_color;
                        }
                    }
                }
            }
        }
    }
}

fn target_px_in_bounds(target_px: (i32, i32), buffer_width: u32, buffer_height: u32) -> bool {
    target_px.0 >= 0
        && target_px.0 < buffer_width as i32
        && target_px.1 >= 0
        && target_px.1 < buffer_height as i32
}
