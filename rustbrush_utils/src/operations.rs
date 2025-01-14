use ecolor::Color32;

use crate::{Brush, ALPHA_CHANNEL};

pub struct PaintOperation<'a> {
    pub pixel_buffer: &'a mut Vec<Color32>,
    pub canvas_width: u32,
    pub canvas_height: u32,
    pub brush: &'a Brush,
    pub color: Color32,
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

        let stamp = self.brush.compute_stamp(self.color);

        for i in 0..=steps {
            let t = i as f32 / steps as f32;
            let x = x0 + dx * t;
            let y = y0 + dy * t;

            for pixel in &stamp.pixels {
                let px = (x + pixel.x as f32) as i32;
                let py = (y + pixel.y as f32) as i32;

                if target_px_in_bounds((px, py), self.canvas_width, self.canvas_height) {
                    let index = (py * self.canvas_width as i32 + px) as usize;
                    let current_color = self.pixel_buffer[index];
                    let stamp_alpha = pixel.color.a() as f32 / 255.0;
                    let blend_strength = stamp_alpha * self.brush.opacity();

                    if self.is_eraser {
                        let new_alpha = current_color.a() as f32 * (1.0 - blend_strength);
                        self.pixel_buffer[index] = Color32::from_rgba_unmultiplied(
                            current_color.r(),
                            current_color.g(),
                            current_color.b(),
                            new_alpha as u8,
                        );
                    } else {
                        self.pixel_buffer[index] = blend_color32(current_color, pixel.color, blend_strength);
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

        // Use whatever color for stamp as we only care about the alpha values (for now?)
        let stamp = self.brush.compute_stamp(Color32::WHITE);

        for i in 0..=steps {
            let t = i as f32 / steps as f32;
            let x = x0 + dx * t;
            let y = y0 + dy * t;

            for pixel in &stamp.pixels {
                let px = (x + pixel.x as f32) as i32;
                let py = (y + pixel.y as f32) as i32;

                if target_px_in_bounds((px, py), self.pixel_buffer_width, self.pixel_buffer_height)
                {
                    let smudge_dx = -dx * self.smudge_strength;
                    let smudge_dy = -dy * self.smudge_strength;

                    let target_px = (px as f32 + smudge_dx) as i32;
                    let target_py = (py as f32 + smudge_dy) as i32;

                    if target_px_in_bounds(
                        (target_px, target_py),
                        self.pixel_buffer_width,
                        self.pixel_buffer_height,
                    ) {
                        let stamp_alpha = pixel.color.a() as f32 / 255.0;
                        let smudge_strength = stamp_alpha * self.smudge_strength;

                        if smudge_strength > 0.0 {
                            let index = (py * self.pixel_buffer_width as i32 + px) as usize;
                            let target_index =
                                (target_py * self.pixel_buffer_width as i32 + target_px) as usize;
                            let current_color = self.pixel_buffer[index];
                            let target_color = self.pixel_buffer[target_index];
                            self.pixel_buffer[index] = blend_color32(current_color, target_color, smudge_strength);
                        }
                    }
                }
            }
        }
    }
}

fn blend_color32(current: Color32, target: Color32, t: f32) -> Color32 {

    let blend = |src_c: u8, dst_c: u8| -> u8 {
        let src_color = src_c as f32;
        let dst_color = dst_c as f32;
        let result_color = src_color + (dst_color - src_color) * t;
        result_color as u8
    };

    Color32::from_rgba_premultiplied(
        blend(current.r(), target.r()),
        blend(current.g(), target.g()),
        blend(current.b(), target.b()),
        blend(current.a(), target.a()),
    )
}

fn target_px_in_bounds(target_px: (i32, i32), buffer_width: u32, buffer_height: u32) -> bool {
    target_px.0 >= 0
        && target_px.0 < buffer_width as i32
        && target_px.1 >= 0
        && target_px.1 < buffer_height as i32
}
