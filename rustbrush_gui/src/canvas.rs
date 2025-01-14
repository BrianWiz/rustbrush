use crate::user::{BrushStrokeFrame, BrushStrokeKind};
use eframe::egui::{self, Color32};
use image::{ImageBuffer, Rgba};
use rustbrush_utils::operations::{PaintOperation, SmudgeOperation};

#[derive(Clone)]
pub struct CanvasLayer {
    pixels: Vec<Color32>,
    pub texture: Option<egui::TextureHandle>,
    pub visible: bool,
    pub name: String,
    dirty: bool,
}

impl CanvasLayer {
    pub fn new(width: u32, height: u32, name: String) -> Self {
        Self {
            pixels: vec![Color32::TRANSPARENT; width as usize * height as usize],
            texture: None,
            visible: true,
            name,
            dirty: true,
        }
    }

    pub fn mark_dirty(&mut self) {
        self.dirty = true;
    }

    pub fn mark_clean(&mut self) {
        self.dirty = false;
    }

    pub fn is_dirty(&self) -> bool {
        self.dirty
    }

    pub fn pixels(&self) -> &Vec<Color32> {
        &self.pixels
    }
}

pub struct CanvasState {
    pub layers: Vec<CanvasLayer>,
    pub width: u32,
    pub height: u32,
}

pub struct Canvas {
    pub state: CanvasState,
}

impl Canvas {
    pub fn process_brush_stroke_frame(
        &mut self,
        layer: usize,
        kind: BrushStrokeKind,
        frame: &BrushStrokeFrame,
    ) {
        match kind {
            BrushStrokeKind::Paint => self.paint(layer, &frame),
            BrushStrokeKind::Erase => self.erase(layer, &frame),
            BrushStrokeKind::Smudge => self.smudge(layer, &frame),
        }
    }

    pub fn clear(&mut self) {
        for layer in self.state.layers.iter_mut() {
            layer.pixels.fill(Color32::TRANSPARENT);
            layer.mark_dirty();
        }
    }

    pub fn clear_layer(&mut self, layer: usize) {
        if let Some(layer) = self.layers().get_mut(layer) {
            layer.pixels.fill(Color32::TRANSPARENT);
            layer.mark_dirty();
        }
    }

    pub fn add_layer(&mut self) {
        let width = self.state.width;
        let height = self.state.height;
        let layer_num = self.layers().len() + 1;
        self.layers().push(CanvasLayer::new(
            width,
            height,
            format!("Layer {}", layer_num),
        ));
    }

    pub fn layers(&mut self) -> &mut Vec<CanvasLayer> {
        &mut self.state.layers
    }

    pub fn save_as_png(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let width = self.state.width;
        let height = self.state.height;

        let mut merged = vec![0u8; (width * height * 4) as usize];

        for layer in self.state.layers.iter() {
            for (i, pixel) in layer.pixels.iter().enumerate() {
                let rgba = Rgba([pixel.r(), pixel.g(), pixel.b(), pixel.a()]);
                merged[i * 4] = rgba[0];
                merged[i * 4 + 1] = rgba[1];
                merged[i * 4 + 2] = rgba[2];
                merged[i * 4 + 3] = rgba[3];
            }
        }

        let image_buffer: ImageBuffer<Rgba<u8>, Vec<u8>> =
            ImageBuffer::from_raw(width, height, merged).ok_or("Failed to create image buffer")?;

        image_buffer.save(path)?;
        Ok(())
    }

    fn paint(&mut self, layer: usize, frame: &BrushStrokeFrame) {
        self.layers()[layer].mark_dirty();
        PaintOperation {
            brush: &frame.brush,
            color: frame.color,
            cursor_position: (frame.cursor_position.x, frame.cursor_position.y),
            last_cursor_position: (frame.last_cursor_position.x, frame.last_cursor_position.y),
            is_eraser: false,
            pixel_buffer: &mut self.state.layers[layer].pixels,
            canvas_width: self.state.width,
            canvas_height: self.state.height,
        }
        .process();
    }

    fn erase(&mut self, layer: usize, frame: &BrushStrokeFrame) {
        self.layers()[layer].mark_dirty();
        PaintOperation {
            brush: &frame.brush,
            color: egui::Rgba::WHITE,
            cursor_position: (frame.cursor_position.x, frame.cursor_position.y),
            last_cursor_position: (frame.last_cursor_position.x, frame.last_cursor_position.y),
            is_eraser: true,
            pixel_buffer: &mut self.state.layers[layer].pixels,
            canvas_width: self.state.width,
            canvas_height: self.state.height,
        }
        .process();
    }

    fn smudge(&mut self, layer: usize, frame: &BrushStrokeFrame) {
        self.layers()[layer].mark_dirty();
        SmudgeOperation {
            brush: &frame.brush,
            cursor_position: (frame.cursor_position.x, frame.cursor_position.y),
            last_cursor_position: (frame.last_cursor_position.x, frame.last_cursor_position.y),
            smudge_strength: 1.0, // @todo: doesn't belong here, infact can probably just use opacity
            pixel_buffer: &mut self.state.layers[layer].pixels,
            pixel_buffer_width: self.state.width,
            pixel_buffer_height: self.state.height,
        }
        .process();
    }
}
