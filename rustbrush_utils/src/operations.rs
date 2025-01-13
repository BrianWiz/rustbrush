use ecolor::Color32;

use crate::Brush;

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

                if target_px_in_bounds((px, py), self.canvas_width, self.canvas_height) {
                    // @todo: !!!DO NOT DELETE THIS COMMENT!!!
                    // It gives a really neat 3D effect to it that I want to officially support

                    //     let index = (py * self.canvas_width as i32 + px) as usize;
                    //     let stamp_alpha = pixel.color[3] as f32 / 255.0;
                    //     let stamp_color = Color32::from_rgba_unmultiplied(
                    //         pixel.color[0],
                    //         pixel.color[1],
                    //         pixel.color[2],
                    //         pixel.color[3],
                    //     );

                    //     if self.is_eraser {
                    //         let current_alpha = self.pixel_buffer[index].a() as f32 / 255.0;
                    //         let erase_strength = stamp_alpha * self.brush.opacity();
                    //         let new_alpha = current_alpha * (1.0 - erase_strength);
                    //         self.pixel_buffer[index] = Color32::from_rgba_unmultiplied(
                    //             self.pixel_buffer[index].r(),
                    //             self.pixel_buffer[index].g(),
                    //             self.pixel_buffer[index].b(),
                    //             (new_alpha * 255.0) as u8,
                    //         );
                    //     } else {
                    //         let src_alpha = stamp_alpha;
                    //         let dst_alpha = self.pixel_buffer[index].a() as f32 / 255.0;
                    //         let result_alpha = src_alpha + dst_alpha * (1.0 - src_alpha);

                    //         if result_alpha > 0.0 {
                    //             let blend = |src_c: u8, dst_c: u8| -> u8 {
                    //                 let src_color = src_c as f32 / 255.0;
                    //                 let dst_color = dst_c as f32 / 255.0;
                    //                 let result_color = (src_color * src_alpha
                    //                     + dst_color * dst_alpha * (1.0 - src_alpha))
                    //                     / result_alpha;
                    //                 (result_color * 255.0) as u8
                    //             };

                    //             self.pixel_buffer[index] = Color32::from_rgba_unmultiplied(
                    //                 blend(stamp_color.r(), self.pixel_buffer[index].r()),
                    //                 blend(stamp_color.g(), self.pixel_buffer[index].g()),
                    //                 blend(stamp_color.b(), self.pixel_buffer[index].b()),
                    //                 (result_alpha * 255.0) as u8,
                    //             );
                    //         }
                    //     }

                    let index = (py * self.canvas_width as i32 + px) as usize;
                    let stamp_alpha = pixel.color[3] as f32 / 255.0;
                    let src_alpha = stamp_alpha;
                    let dst_alpha = self.pixel_buffer[index].a() as f32 / 255.0;
                    let result_alpha = src_alpha + dst_alpha * (1.0 - src_alpha);

                    if result_alpha > 0.0 {
                        let blend = |src_c: u8, dst_c: u8| -> u8 {
                            let src_color = src_c as f32;
                            let dst_color = dst_c as f32;
                            let result_color =
                                src_color * src_alpha + dst_color * (1.0 - src_alpha);
                            result_color as u8
                        };

                        self.pixel_buffer[index] = Color32::from_rgba_unmultiplied(
                            blend(pixel.color[0], self.pixel_buffer[index].r()),
                            blend(pixel.color[1], self.pixel_buffer[index].g()),
                            blend(pixel.color[2], self.pixel_buffer[index].b()),
                            (result_alpha * 255.0) as u8,
                        );
                    }
                }
            }
        }
    }
}

pub struct SmearOperation<'a> {
    pub pixel_buffer: &'a mut Vec<Color32>,
    pub pixel_buffer_width: u32,
    pub pixel_buffer_height: u32,
    pub brush: &'a Brush,
    pub cursor_position: (f32, f32),
    pub last_cursor_position: (f32, f32),
    pub smear_strength: f32,
}

fn lerp(a: u8, b: u8, t: f32) -> u8 {
    let start = a as f32;
    let end = b as f32;
    ((start + (end - start) * t).round() as u8).clamp(0, 255)
}

fn lerp_color32(c1: Color32, c2: Color32, t: f32) -> Color32 {
    Color32::from_rgba_unmultiplied(
        lerp(c1.r(), c2.r(), t),
        lerp(c1.g(), c2.g(), t),
        lerp(c1.b(), c2.b(), t),
        lerp(c1.a(), c2.a(), t),
    )
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

        // Use white color for stamp as we only care about the alpha values
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
                    let smear_dx = -dx * self.smear_strength;
                    let smear_dy = -dy * self.smear_strength;

                    let target_px = (px as f32 + smear_dx) as i32;
                    let target_py = (py as f32 + smear_dy) as i32;

                    if target_px_in_bounds(
                        (target_px, target_py),
                        self.pixel_buffer_width,
                        self.pixel_buffer_height,
                    ) {
                        let stamp_alpha = pixel.color[3] as f32 / 255.0;
                        let smear_strength = stamp_alpha * self.smear_strength;

                        if smear_strength > 0.0 {
                            let index = (py * self.pixel_buffer_width as i32 + px) as usize;
                            let target_index =
                                (target_py * self.pixel_buffer_width as i32 + target_px) as usize;

                            let current_color = self.pixel_buffer[index];
                            let target_color = self.pixel_buffer[target_index];

                            self.pixel_buffer[index] =
                                lerp_color32(current_color, target_color, smear_strength);
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
