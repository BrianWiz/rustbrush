mod canvas;
mod user;

use canvas::{Canvas, CanvasLayer, CanvasState};
use eframe::egui::{self, Color32, Pos2, Rect, Vec2};
use tracing::error;
use user::User;

struct ViewState {
    offset: Vec2,
    zoom: f32,
}

impl Default for ViewState {
    fn default() -> Self {
        Self {
            offset: Vec2::ZERO,
            zoom: 1.0,
        }
    }
}

struct PixelPainter {
    canvas: Canvas,
    active_layer: usize,
    view: ViewState,
    dragging_canvas: bool,
    last_drag_pos: Option<Pos2>,
    user: User,
}

impl Default for PixelPainter {
    fn default() -> Self {
        let width = 800;
        let height = 600;
        let mut layers = Vec::new();
        layers.push(CanvasLayer::new(width, height, "Background".to_string()));
        layers.push(CanvasLayer::new(width, height, "Layer 1".to_string()));

        Self {
            canvas: Canvas {
                state: CanvasState {
                    layers,
                    width: width as u32,
                    height: height as u32,
                },
            },
            active_layer: 1,
            view: ViewState::default(),
            dragging_canvas: false,
            last_drag_pos: None,
            user: User::default(),
        }
    }
}

impl PixelPainter {
    fn screen_to_canvas(&self, screen_pos: Pos2, canvas_rect: Rect) -> Pos2 {
        let relative_pos = screen_pos - canvas_rect.min - self.view.offset;
        Pos2::new(
            relative_pos.x / self.view.zoom,
            relative_pos.y / self.view.zoom,
        )
    }
}

impl eframe::App for PixelPainter {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let width = self.canvas.state.width as usize;
        let height = self.canvas.state.height as usize;
        for layer in self.canvas.layers().iter_mut() {
            if layer.is_dirty() || layer.texture.is_none() {
                layer.texture = Some(ctx.load_texture(
                    "layer_texture",
                    egui::ColorImage {
                        size: [width, height],
                        pixels: layer.pixels().clone(),
                    },
                    egui::TextureOptions::default(),
                ));
                layer.mark_clean();
            }
        }

        // Top panel
        let mut new_brush_radius = self.user.current_paint_brush.radius();
        let mut new_brush_color = self.user.current_color;
        let mut canvas_rect = Rect::NOTHING;

        egui::TopBottomPanel::top("controls").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("Pixel Painter");
                ui.separator();
                if ui.button("Clear Layer").clicked() {
                    self.canvas.clear_layer(self.active_layer);
                }
                if ui.button("Add Layer").clicked() {
                    self.canvas.add_layer();
                }
                ui.add(egui::Slider::new(&mut new_brush_radius, 1.0..=20.0).text("Brush Size"));
                ui.color_edit_button_srgba(&mut new_brush_color);
                ui.separator();
                ui.label("View:");
                if ui.button("Reset View").clicked() {
                    self.view = ViewState::default();
                }
                ui.add(egui::Slider::new(&mut self.view.zoom, 0.1..=10.0).text("Zoom"));
            });
        });

        // Layer panel
        egui::SidePanel::left("layers").show(ctx, |ui| {
            ui.heading("Layers");
            ui.separator();

            for (i, layer) in self.canvas.layers().iter_mut().enumerate().rev() {
                ui.horizontal(|ui| {
                    ui.checkbox(&mut layer.visible, "");
                    if ui
                        .selectable_label(self.active_layer == i, &layer.name)
                        .clicked()
                    {
                        self.active_layer = i;
                    }
                });
            }
        });

        // Main canvas area
        egui::CentralPanel::default().show(ctx, |ui| {
            let available_size = ui.available_size();
            canvas_rect = Rect::from_min_size(ui.cursor().min, available_size);

            // Handle canvas panning
            let response = ui.allocate_rect(canvas_rect, egui::Sense::drag());
            if response.dragged_by(egui::PointerButton::Middle) {
                if self.last_drag_pos.is_some() {
                    let delta = response.drag_delta();
                    self.view.offset += delta;
                }
                self.dragging_canvas = true;
                self.last_drag_pos = Some(response.hover_pos().unwrap_or_default());
            } else {
                self.dragging_canvas = false;
                self.last_drag_pos = None;
            }

            // Handle scroll for zoom
            if let Some(hover_pos) = response.hover_pos() {
                let zoom_delta = ui.input(|i| i.raw_scroll_delta.y / 200.0);
                if zoom_delta != 0.0 {
                    let old_zoom = self.view.zoom;
                    self.view.zoom = (self.view.zoom * (1.0 + zoom_delta)).clamp(0.1, 10.0);

                    let zoom_center = hover_pos - canvas_rect.min - self.view.offset;
                    let zoom_factor = self.view.zoom / old_zoom;
                    let new_center = zoom_center * zoom_factor;
                    self.view.offset += zoom_center - new_center;
                }
            }

            // Draw all visible layers
            let texture_size = Vec2::new(
                self.canvas.state.width as f32 * self.view.zoom,
                self.canvas.state.height as f32 * self.view.zoom,
            );

            for layer in self.canvas.layers().iter().filter(|l| l.visible) {
                if let Some(texture) = &layer.texture {
                    ui.painter().image(
                        texture.id(),
                        Rect::from_min_size(canvas_rect.min + self.view.offset, texture_size),
                        Rect::from_min_max(Pos2::ZERO, Pos2::new(1.0, 1.0)),
                        Color32::WHITE,
                    );
                }
            }
        });

        // Apply state updates
        self.user.current_paint_brush.set_radius(new_brush_radius);
        self.user.current_color = new_brush_color;

        // Handle painting
        if let Some(pointer_pos) = ctx.pointer_hover_pos() {
            if !self.dragging_canvas {
                self.user.cursor_position = self.screen_to_canvas(pointer_pos, canvas_rect);

                if ctx.input(|i| i.pointer.primary_pressed()) {
                    self.user.holding_pointer_primary = true;
                    self.user.start_brush_stroke(user::BrushStrokeKind::Paint);
                }

                if ctx.input(|i| i.pointer.primary_released()) {
                    self.user.holding_pointer_primary = false;
                }

                if ctx.input(|i| i.pointer.secondary_pressed()) {
                    self.user.start_brush_stroke(user::BrushStrokeKind::Smudge);
                    self.user.holding_pointer_right = true;
                }

                if ctx.input(|i| i.pointer.secondary_released()) {
                    self.user.holding_pointer_right = false;
                }

                if self.user.holding_pointer_primary || self.user.holding_pointer_right {
                    match self.user.continue_brush_stroke() {
                        Ok((layer_idx, brush_stroke_kind, brush_stroke_frame)) => {
                            self.canvas.process_brush_stroke_frame(
                                layer_idx,
                                brush_stroke_kind,
                                brush_stroke_frame,
                            );
                        }
                        Err(e) => error!("Error processing brush stroke: {:?}", e),
                    }
                }

                self.user.last_cursor_position = self.user.cursor_position;
            }
        }
    }
}

fn main() -> Result<(), eframe::Error> {
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "Pixel Painter",
        native_options,
        Box::new(|_cc| Ok(Box::new(PixelPainter::default()))),
    )
}
