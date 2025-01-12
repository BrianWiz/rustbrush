use std::sync::Arc;

use tracing::error;
use winit::window::Window;
use pixels::{Pixels, SurfaceTexture};

use rustbrush_utils::ALPHA_CHANNEL;
use crate::canvas::{Canvas, CanvasState};

pub struct RenderState {
    pub pixels: Pixels<'static>,
    pub canvas: Canvas,
}

impl RenderState {
    pub fn new(window: Arc<Window>, width: u32, height: u32) -> Self {
        let surface_size = window.inner_size();
        let surface_texture = SurfaceTexture::new(surface_size.width, surface_size.height, window);

        // transparent layers by default
        let layer_size = (surface_size.width * surface_size.height * 4) as usize;
        let layer1 = vec![0u8; layer_size];
        let layer2 = vec![0u8; layer_size];

        let pixels = Pixels::new(
            width,
            height,
            surface_texture,
        )
            .expect("Failed to create pixels. Cannot continue.");

        Self {
            pixels,
            canvas: Canvas {
                state: CanvasState {
                    layers: vec![layer1, layer2],
                    width,
                    height,
                },
                current_layer: 0,
                dirty: true,
            }
        }
    }

    pub fn render(&mut self) -> Result<(), Box<dyn std::error::Error>> {

        if !self.canvas.dirty {
            return Ok(());
        }

        self.canvas.dirty = false;

        let frame = self.pixels.frame_mut();
        frame.fill(0); // clears the frame

        // merge layers into the frame
        for layer in self.canvas.layers() {
            for (i, chunk) in frame.chunks_mut(4).enumerate() {
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
        
        self.pixels.render()?;
        Ok(())
    }

    pub fn resize_surface(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            if let Err(e) = self.pixels.resize_surface(width, height) {
                error!("Failed to resize surface: {:?}", e);
            }
        }
    }
}
