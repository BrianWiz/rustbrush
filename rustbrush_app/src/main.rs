mod app;
mod user;
mod canvas;
mod render;

use app::App;
use tracing_subscriber::{filter::LevelFilter, EnvFilter};
use winit::event_loop::{ControlFlow, EventLoop};


fn main() -> Result<(), Box<dyn std::error::Error>> {

    let filter = EnvFilter::builder()
        .with_default_directive(LevelFilter::INFO.into())
        .parse("wgpu=warn")?
        .add_directive("rustbrush_app=info".parse()?);

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .init();

    let event_loop = EventLoop::new()?;
    event_loop.set_control_flow(ControlFlow::Wait);

    let mut app = App::default();
    event_loop.run_app(&mut app)?;

    Ok(())
}