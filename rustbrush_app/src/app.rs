use std::sync::Arc;

use tracing::error;
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::ActiveEventLoop,
    window::{Window, WindowId},
};
use crate::render::state::RenderState;

#[derive(Default)]
pub struct App {
    window: Option<Arc<Window>>,
    render_state: Option<RenderState>,
    cursor_position: (f32, f32),
    last_cursor_position: (f32, f32),
    holding_mouse_left: bool,
    holding_mouse_right: bool,
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = event_loop.create_window(
            Window::default_attributes()
                .with_title("Brushy")
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
                if let Some(render_state) = &mut self.render_state {
                    if self.holding_mouse_left {
                        render_state.canvas.paint(self.cursor_position, self.last_cursor_position);
                    } else if self.holding_mouse_right {
                        render_state.canvas.smudge(self.cursor_position, self.last_cursor_position);
                        //render_state.canvas.erase(self.cursor_position, self.last_cursor_position);
                    }
                    match render_state.render() {
                        Ok(_) => {},
                        Err(e) => error!("Error rendering frame: {:?}", e),
                    }
                }
                self.last_cursor_position = self.cursor_position;
                if let Some(window) = &self.window {
                    window.request_redraw();
                }
            },
            WindowEvent::CursorMoved { position, .. } => {
                self.cursor_position = (position.x as f32, position.y as f32);
            },
            WindowEvent::MouseInput { state, button, .. } => {
                match state {
                    winit::event::ElementState::Pressed => {
                        if button == winit::event::MouseButton::Left {
                            self.holding_mouse_left = true;
                        } else if button == winit::event::MouseButton::Right {
                            self.holding_mouse_right = true;
                        }
                    },
                    winit::event::ElementState::Released => {
                        if button == winit::event::MouseButton::Left {
                            self.holding_mouse_left = false;
                        } else if button == winit::event::MouseButton::Right {
                            self.holding_mouse_right = false;
                        }
                    },
                }
            },
            WindowEvent::KeyboardInput {
                ..
            } => {
            },
            _ => {},
        }
    }
}
