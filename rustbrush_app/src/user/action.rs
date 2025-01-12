use std::time::Instant;

use rustbrush_utils::Brush;

pub struct User {
    pub current_color: [u8; 3],
    pub current_paint_brush: Brush,
    pub current_eraser_brush: Brush,
    pub current_smudge_brush: Brush,
    pub cursor_position: (f32, f32),
    pub last_cursor_position: (f32, f32),
    pub holding_mouse_left: bool,
    pub holding_mouse_right: bool,
    pub action_history: Vec<UserAction>,
}

impl Default for User {
    fn default() -> Self {
        Self {
            current_color: [255, 255, 255],
            current_paint_brush: Brush::default().with_opacity(0.5),
            current_eraser_brush: Brush::default().with_opacity(0.5),
            current_smudge_brush: Brush::default().with_opacity(0.5),
            cursor_position: (0.0, 0.0),
            last_cursor_position: (0.0, 0.0),
            holding_mouse_left: false,
            holding_mouse_right: false,
            action_history: Vec::new(),
        }
    }
}

impl User {
    pub fn start_brush_stroke(&mut self, kind: BrushStrokeKind) {
        self.action_history.push(UserAction::BrushStroke(BrushStroke::new(kind)));
    }

    pub fn continue_brush_stroke(&mut self) -> Result<(BrushStrokeKind, &BrushStrokeFrame), Box<dyn std::error::Error>> {
        let color = self.current_color;

        let current_action_kind: BrushStrokeKind = match self.current_action() {
            Some(UserAction::BrushStroke(stroke)) => stroke.kind.clone(),
            None => return Err("No current action! Make sure you call start_brush_stroke()".into()),
        };

        let brush = match current_action_kind {
            BrushStrokeKind::Paint => {
                self.current_paint_brush.clone()
            },
            BrushStrokeKind::Erase => {
                self.current_eraser_brush.clone()
            },
            BrushStrokeKind::Smudge => {
                self.current_smudge_brush.clone()
            },
        };

        let cursor_position = self.cursor_position;
        let last_cursor_position = self.last_cursor_position;

        if let Some(UserAction::BrushStroke(stroke)) = self.current_action() {
            stroke.add_frame(BrushStrokeFrame {
                timestamp: Instant::now(),
                brush,
                color,
                cursor_position,
                last_cursor_position,
            });

            return Ok((current_action_kind, stroke.frames.last().unwrap()));
        }

        Err("I have absolutely no idea how you ended up here. You will have to read the code, sorry.".into())
    }

    fn current_action(&mut self) -> Option<&mut UserAction> {
        self.action_history.last_mut()
    }
}

pub enum UserAction {
    BrushStroke(BrushStroke),
}

#[derive(Clone)]
pub enum BrushStrokeKind {
    Paint,
    Erase,
    Smudge,
}

pub struct BrushStroke {
    pub kind: BrushStrokeKind,
    pub frames: Vec<BrushStrokeFrame>,
}

impl BrushStroke {
    pub fn new(kind: BrushStrokeKind) -> Self {
        Self {
            kind,
            frames: Vec::new(),
        }
    }

    pub fn add_frame(&mut self, frame: BrushStrokeFrame) {
        self.frames.push(frame);
    }
}

pub struct BrushStrokeFrame {
    pub timestamp: Instant,
    pub brush: Brush,
    pub color: [u8; 3],
    pub cursor_position: (f32, f32),
    pub last_cursor_position: (f32, f32),
}
