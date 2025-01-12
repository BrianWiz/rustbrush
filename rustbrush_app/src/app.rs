use std::{sync::Arc, time::Instant};

use tracing::{error, info};
use winit::{
    application::ApplicationHandler, event::{ElementState, KeyEvent, WindowEvent}, event_loop::ActiveEventLoop, keyboard::{KeyCode, PhysicalKey}, platform::windows::{Color, WindowAttributesExtWindows}, window::{Window, WindowId}
};
use crate::{render::state::RenderState, user::user::{BrushStrokeKind, User}};

#[derive(Default)]
pub struct App {
    window: Option<Arc<Window>>,
    render_state: Option<RenderState>,
    user: User,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = event_loop.create_window(
            Window::default_attributes()
                .with_title("Brushy")
                .with_title_background_color(Some(Color::from_rgb(0,0,0))),
        ).expect("Failed to create window");

        let window = Arc::new(window);
        self.window = Some(window.clone());
        self.render_state = Some(RenderState::new(window, 800, 600));
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            },
            WindowEvent::Resized(new_size) => {
                if let Some(render_state) = &mut self.render_state {
                    render_state.resize_surface(new_size.width, new_size.height);
                }
            },
            WindowEvent::RedrawRequested => {
                let brush_stroke_frame = if self.user.holding_mouse_left || self.user.holding_mouse_right {
                    match self.user.continue_brush_stroke() {
                        Ok(frame) => Some(frame),
                        Err(e) => {
                            error!("Error continuing brush stroke: {:?}", e);
                            None
                        }
                    }
                } else {
                    None
                };

                if let Some(render_state) = &mut self.render_state {

                    if let Some(frame_layer_and_kind) = brush_stroke_frame {
                        render_state.canvas.process_brush_stroke_frame(frame_layer_and_kind.0, frame_layer_and_kind.1, frame_layer_and_kind.2);
                    }

                    match render_state.render() {
                        Ok(_) => {},
                        Err(e) => error!("Error rendering frame: {:?}", e),
                    }
                }
                self.user.last_cursor_position = self.user.cursor_position;
                if let Some(window) = &self.window {
                    window.request_redraw();
                }
            },
            WindowEvent::CursorMoved { position, .. } => {
                self.user.cursor_position = (position.x as f32, position.y as f32);
            },
            WindowEvent::MouseInput { state, button, .. } => {
                match state {
                    winit::event::ElementState::Pressed => {
                        if button == winit::event::MouseButton::Left {
                            self.user.holding_mouse_left = true;
                            self.user.start_brush_stroke(BrushStrokeKind::Paint);
                        } else if button == winit::event::MouseButton::Right {
                            self.user.holding_mouse_right = true;
                            self.user.start_brush_stroke(BrushStrokeKind::Smudge);
                        }
                    },
                    winit::event::ElementState::Released => {
                        if button == winit::event::MouseButton::Left {
                            self.user.holding_mouse_left = false;
                        } else if button == winit::event::MouseButton::Right {
                            self.user.holding_mouse_right = false;
                        }
                    },
                }
            },
            WindowEvent::ModifiersChanged(modifiers) => {
                self.user.holding_ctrl = modifiers.state().control_key();
            },
            WindowEvent::KeyboardInput { 
                event: KeyEvent { 
                    physical_key,
                    state: ElementState::Pressed,
                    ..
                },
                ..
            } => {
                if let Some(render_state) = &mut self.render_state {
                    let mut canvas = &mut render_state.canvas;
                    if self.user.holding_ctrl && physical_key == PhysicalKey::Code(KeyCode::KeyZ) {
                        self.user.undo(&mut canvas);
                    } else if self.user.holding_ctrl && physical_key == PhysicalKey::Code(KeyCode::KeyY) {
                        self.user.redo(&mut canvas);
                    } else if self.user.holding_ctrl && physical_key == PhysicalKey::Code(KeyCode::KeyS) {
                        let file_name = chrono::Local::now().format("brushy_%Y-%m-%d_%H-%M-%S.png").to_string();
                        match canvas.save_as_png(file_name.as_str()) {
                            Ok(_) => info!("Saved as {}", file_name),
                            Err(e) => error!("Error saving: {:?}", e),
                        }
                    }
                }
            },
            _ => {},
        }
    }
}
