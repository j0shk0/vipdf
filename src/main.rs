mod parser;
mod renderer;
mod app;

use winit::event_loop::{ ControlFlow, EventLoop};
use app::App;

fn main() {
    let file = std::fs::read(std::env::args().nth(1).unwrap()).unwrap();

    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Wait);

    let mut app = App::new(file);
    event_loop.run_app(&mut app).unwrap();
}
