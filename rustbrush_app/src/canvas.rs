use rustbrush_utils::{operations::{PaintOperation, SmearOperation}, Brush};

pub struct Canvas {
    pub layers: Vec<Vec<u8>>,
    pub width: u32,
    pub height: u32,
    pub current_layer: usize,
    pub dirty: bool,
}

impl Canvas {
    pub fn paint(&mut self, cursor_position: (f32, f32), last_cursor_position: (f32, f32)) {
        self.dirty = true;
        PaintOperation {
            pixel_buffer: &mut self.layers[self.current_layer],
            brush: &Brush::default().with_opacity(0.5),
            color: [255, 255, 255],
            pixel_buffer_width: self.width,
            pixel_buffer_height: self.height,
            cursor_position,
            last_cursor_position,
            is_eraser: false,
        }
            .process();
    }

    pub fn erase(&mut self, cursor_position: (f32, f32), last_cursor_position: (f32, f32)) {
        self.dirty = true;
        PaintOperation {
            pixel_buffer: &mut self.layers[self.current_layer],
            pixel_buffer_width: self.width,
            pixel_buffer_height: self.height,
            brush: &Brush::default().with_opacity(0.5),
            color: [0, 0, 0], // doesn't even get used for eraser so doesn't matter
            cursor_position,
            last_cursor_position,
            is_eraser: true,
        }
            .process();
    }

    pub fn smudge(&mut self, cursor_position: (f32, f32), last_cursor_position: (f32, f32)) {
        self.dirty = true;
        SmearOperation {
            pixel_buffer: &mut self.layers[self.current_layer],
            pixel_buffer_width: self.width,
            pixel_buffer_height: self.height,
            brush: &Brush::default().with_opacity(0.5),
            cursor_position,
            last_cursor_position,
            smear_strength: 1.0,
        }
            .process();
    }
}
