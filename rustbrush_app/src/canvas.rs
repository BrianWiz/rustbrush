use rustbrush_utils::{operations::{PaintOperation, SmearOperation}, Brush};

use crate::user::action::{BrushStrokeFrame, BrushStrokeKind};

pub struct CanvasState {
    pub layers: Vec<Vec<u8>>,
    pub width: u32,
    pub height: u32,
}

pub struct Canvas {
    pub state: CanvasState,
    pub current_layer: usize,
    pub dirty: bool,
}

impl Canvas {
    pub fn process_brush_stroke_frame(&mut self, kind: BrushStrokeKind, frame: &BrushStrokeFrame) {
        match kind {
            BrushStrokeKind::Paint => self.paint(&frame),
            BrushStrokeKind::Erase => self.erase(&frame),
            BrushStrokeKind::Smudge => self.smudge(&frame),
        }
    }

    fn paint(&mut self, frame: &BrushStrokeFrame) {
        self.dirty = true;
        PaintOperation {
            brush: &frame.brush,
            color: frame.color,
            cursor_position: frame.cursor_position,
            last_cursor_position: frame.last_cursor_position,
            is_eraser: false,
            pixel_buffer: &mut self.state.layers[self.current_layer],
            pixel_buffer_width: self.state.width,
            pixel_buffer_height: self.state.height,
        }
            .process();
    }

    fn erase(&mut self, frame: &BrushStrokeFrame) {
        self.dirty = true;
        PaintOperation {
            brush: &frame.brush,
            color: [0, 0, 0], // doesn't even get used for eraser so doesn't matter
            cursor_position: frame.cursor_position,
            last_cursor_position: frame.last_cursor_position,
            is_eraser: true,
            pixel_buffer: &mut self.state.layers[self.current_layer],
            pixel_buffer_width: self.state.width,
            pixel_buffer_height: self.state.height,
        }
            .process();
    }

    fn smudge(&mut self, frame: &BrushStrokeFrame) {
        self.dirty = true;
        SmearOperation {
            brush: &frame.brush,
            cursor_position: frame.cursor_position,
            last_cursor_position: frame.last_cursor_position,
            smear_strength: 1.0, // @todo: doesn't belong here
            pixel_buffer: &mut self.state.layers[self.current_layer],
            pixel_buffer_width: self.state.width,
            pixel_buffer_height: self.state.height,
        }
            .process();
    }

    pub fn layers(&self) -> &[Vec<u8>] {
        &self.state.layers
    }
}
