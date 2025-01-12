use image::{ImageBuffer, Rgba};
use rustbrush_utils::{operations::{PaintOperation, SmearOperation}, ALPHA_CHANNEL};

use crate::user::user::{BrushStrokeFrame, BrushStrokeKind};

pub struct CanvasState {
    pub layers: Vec<Vec<u8>>,
    pub width: u32,
    pub height: u32,
}

pub struct Canvas {
    pub state: CanvasState,
    pub dirty: bool,
}

impl Canvas {
    pub fn process_brush_stroke_frame(&mut self, layer: usize, kind: BrushStrokeKind, frame: &BrushStrokeFrame) {
        match kind {
            BrushStrokeKind::Paint => self.paint(layer, &frame),
            BrushStrokeKind::Erase => self.erase(layer, &frame),
            BrushStrokeKind::Smudge => self.smudge(layer, &frame),
        }
    }

    pub fn clear(&mut self) {
        self.dirty = true;
        for layer in self.state.layers.iter_mut() {
            layer.iter_mut().for_each(|pixel| *pixel = 0);
        }
    }

    fn paint(&mut self, layer: usize, frame: &BrushStrokeFrame) {
        self.dirty = true;
        PaintOperation {
            brush: &frame.brush,
            color: frame.color,
            cursor_position: frame.cursor_position,
            last_cursor_position: frame.last_cursor_position,
            is_eraser: false,
            pixel_buffer: &mut self.state.layers[layer],
            pixel_buffer_width: self.state.width,
            pixel_buffer_height: self.state.height,
        }
            .process();
    }

    fn erase(&mut self, layer: usize, frame: &BrushStrokeFrame) {
        self.dirty = true;
        PaintOperation {
            brush: &frame.brush,
            color: [0, 0, 0], // doesn't even get used for eraser so doesn't matter
            cursor_position: frame.cursor_position,
            last_cursor_position: frame.last_cursor_position,
            is_eraser: true,
            pixel_buffer: &mut self.state.layers[layer],
            pixel_buffer_width: self.state.width,
            pixel_buffer_height: self.state.height,
        }
            .process();
    }

    fn smudge(&mut self, layer: usize, frame: &BrushStrokeFrame) {
        self.dirty = true;
        SmearOperation {
            brush: &frame.brush,
            cursor_position: frame.cursor_position,
            last_cursor_position: frame.last_cursor_position,
            smear_strength: 1.0, // @todo: doesn't belong here
            pixel_buffer: &mut self.state.layers[layer],
            pixel_buffer_width: self.state.width,
            pixel_buffer_height: self.state.height,
        }
            .process();
    }

    pub fn layers(&self) -> &[Vec<u8>] {
        &self.state.layers
    }

    pub fn save_as_png(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let width = self.state.width;
        let height = self.state.height;
        
        let mut merged = vec![0u8; (width * height * 4) as usize];
        
        for layer in self.layers() {
            for (i, chunk) in merged.chunks_mut(4).enumerate() {
                let layer_pixel = &layer[i * 4..(i + 1) * 4];
                
                let alpha = layer_pixel[3] as f32 / 255.0;
                for c in 0..3 {
                    let existing = chunk[c] as f32 / 255.0;
                    let new = layer_pixel[c] as f32 / 255.0;
                    chunk[c] = ((new * alpha + existing * (1.0 - alpha)) * 255.0) as u8;
                }
                
                let existing_alpha = chunk[ALPHA_CHANNEL] as f32 / 255.0;
                let new_alpha = alpha;
                chunk[ALPHA_CHANNEL] = ((new_alpha + existing_alpha * (1.0 - new_alpha)) * 255.0) as u8;
            }
        }

        let image_buffer: ImageBuffer<Rgba<u8>, Vec<u8>> = ImageBuffer::from_raw(
            width,
            height,
            merged
        ).ok_or("Failed to create image buffer")?;

        image_buffer.save(path)?;
        Ok(())
    }
}
