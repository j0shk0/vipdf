mod app;
mod hayro_renderer;
mod parser;
mod pdfium_renderer;

use app::App;
use winit::event_loop::{ControlFlow, EventLoop};

const USAGE: &str = "\
Usage:
  vipdf [OPTIONS] <FILE>

Options:
  --help        Print this information
  --version     Print version information

Controls:
  gg            Top of the first page
  Shift+g        Top of the last page
  j             Next page or scroll down if page is taller than window
  k             Previous page or scroll up if page is taller than the window
  <number>gg    Jump to page <number> (Careful: might not match table of content)
  +             Zoom in
  -             Zoom out
  Close window  Quit
";

const VERSION: &str = "vipdf v0.3.0";

fn main() {
    let first_argument = &std::env::args().nth(1);

    let file_name = match first_argument {
        Some(text) => {
            if text.eq("--help") {
                println!("{}", USAGE);
                return;
            }
            if text.eq("--version") {
                println!("{}", VERSION);
                return;
            }
            text
        }
        None => {
            println!("Please provide a PDF to open!");
            return;
        }
    };

    let path = std::path::Path::new(file_name);
    if !path.exists() || !file_name.ends_with(".pdf") {
        println!("File does not exist or is not a PDF!");
        return;
    }

    #[cfg(feature = "pdfium")]
    let app_input = file_name.as_str();
    #[cfg(not(feature = "pdfium"))]
    let app_input = std::fs::read(&file_name).unwrap();

    let event_loop = EventLoop::new()
        .expect("EventLoop::new: requires main thread and a running display server");
    event_loop.set_control_flow(ControlFlow::Wait);
    let mut app = App::new(app_input);
    event_loop
        .run_app(&mut app)
        .expect("event loop terminated with an error");
}
