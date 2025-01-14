use std::time::Instant;

use crate::canvas::Canvas;
use eframe::egui::{Color32, Pos2, Rgba};
use rustbrush_utils::Brush;

pub type LayerIdx = usize;

pub struct User {
    pub current_color: Rgba,
    pub current_paint_brush: Brush,
    pub current_eraser_brush: Brush,
    pub current_smudge_brush: Brush,
    pub current_layer: LayerIdx,
    pub current_action_id: usize,
    pub action_history: Vec<UserAction>,

    // all of these are set by the App struct
    pub cursor_position: Pos2,
    pub last_cursor_position: Pos2,
    pub holding_pointer_primary: bool,
    pub holding_pointer_right: bool,
}

impl Default for User {
    fn default() -> Self {
        Self {
            current_color: Rgba::WHITE,
            current_paint_brush: Brush::default().with_strength(1.0),
            current_eraser_brush: Brush::default().with_strength(1.0),
            current_smudge_brush: Brush::default().with_strength(1.0),
            current_layer: 0,
            current_action_id: 0,
            action_history: Vec::new(),

            cursor_position: Pos2::ZERO,
            last_cursor_position: Pos2::ZERO,
            holding_pointer_primary: false,
            holding_pointer_right: false,
        }
    }
}

impl User {
    pub fn undo(&mut self, canvas: &mut Canvas) {
        if self.current_action_id > 0 {
            self.current_action_id -= 1;
            canvas.clear();
            for action in self
                .action_history
                .iter()
                .filter(|a| a.id <= self.current_action_id)
            {
                match &action.data {
                    UserActionData::BrushStroke(stroke) => {
                        for frame in &stroke.frames {
                            canvas.process_brush_stroke_frame(
                                self.current_layer,
                                stroke.kind.clone(),
                                frame,
                            );
                        }
                    }
                }
            }
            canvas.layers()[self.current_layer].mark_dirty();
        }
    }

    pub fn redo(&mut self, canvas: &mut Canvas) {
        if let Some(next_action) = self
            .action_history
            .iter()
            .find(|a| a.id > self.current_action_id)
        {
            self.current_action_id = next_action.id;
            canvas.clear();
            for action in self
                .action_history
                .iter()
                .filter(|a| a.id <= self.current_action_id)
            {
                match &action.data {
                    UserActionData::BrushStroke(stroke) => {
                        for frame in &stroke.frames {
                            canvas.process_brush_stroke_frame(
                                self.current_layer,
                                stroke.kind.clone(),
                                frame,
                            );
                        }
                    }
                }
            }
            canvas.layers()[self.current_layer].mark_dirty();
        }
    }

    pub fn start_brush_stroke(&mut self, kind: BrushStrokeKind) {
        self.truncate_action_history();
        self.current_action_id += 1;
        self.action_history.push(UserAction {
            kind: UserActionKind::BrushStroke,
            id: self.current_action_id,
            timestamp: Instant::now(),
            data: UserActionData::BrushStroke(BrushStroke::new(kind)),
        });
    }

    pub fn continue_brush_stroke(
        &mut self,
    ) -> Result<(LayerIdx, BrushStrokeKind, &BrushStrokeFrame), Box<dyn std::error::Error>> {
        let layer = self.current_layer;
        let color = self.current_color;

        let current_brush_stroke_kind: BrushStrokeKind = match self.current_action() {
            Some(action) => match &action.data {
                UserActionData::BrushStroke(stroke) => stroke.kind.clone(),
                _ => return Err("Current action is not a brush stroke".into()),
            },
            None => return Err("No current action".into()),
        };

        let brush = match current_brush_stroke_kind {
            BrushStrokeKind::Paint => self.current_paint_brush.clone(),
            BrushStrokeKind::Erase => self.current_eraser_brush.clone(),
            BrushStrokeKind::Smudge => self.current_smudge_brush.clone(),
        };

        let cursor_position = self.cursor_position;
        let last_cursor_position = self.last_cursor_position;

        if let Some((layer, current_action_kind, action)) = self
            .current_action()
            .map(|action| (layer, current_brush_stroke_kind, action))
        {
            match &mut action.data {
                UserActionData::BrushStroke(stroke) => {
                    stroke.add_frame(BrushStrokeFrame {
                        brush,
                        color,
                        cursor_position,
                        last_cursor_position,
                    });

                    return Ok((layer, current_action_kind, &stroke.frames.last().unwrap()));
                }
            }
        }

        Err("I have absolutely no idea how you ended up here. You will have to read the code, sorry.".into())
    }

    fn current_action(&mut self) -> Option<&mut UserAction> {
        for action in self.action_history.iter_mut().rev() {
            if action.id == self.current_action_id {
                return Some(action);
            }
        }
        None
    }

    /// Remove all actions from the history that are older than the current action.
    /// This is used for when the user undoes an action and then performs a new action.
    fn truncate_action_history(&mut self) {
        let current_action_id = self.current_action_id;
        self.action_history
            .retain(|action| action.id <= current_action_id);
    }
}

#[derive(Clone)]
pub enum UserActionKind {
    BrushStroke,
}

pub struct UserAction {
    pub id: usize,
    pub kind: UserActionKind,
    pub timestamp: Instant,
    pub data: UserActionData,
}

pub enum UserActionData {
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
    pub brush: Brush,
    pub color: Rgba,
    pub cursor_position: Pos2,
    pub last_cursor_position: Pos2,
}
