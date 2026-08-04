mod parser;
mod hayro_renderer;
mod app;
mod pdfium_renderer;

use winit::event_loop::{ControlFlow, EventLoop};
use app::App;

fn main() {

    #[cfg(feature = "pdfium")]
    let file = &std::env::args().nth(1).unwrap();
    #[cfg(not(feature = "pdfium"))]
    let file = std::fs::read(std::env::args().nth(1).unwrap()).unwrap();

    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Wait);

    let mut app = App::new(file);
    event_loop.run_app(&mut app).unwrap();
}
