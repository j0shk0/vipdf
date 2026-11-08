use crate::event_handler::EventHandler;
use crate::state::AppState;
use crate::window_handler::WindowHandler;
#[cfg(feature = "pdfium")]
use pdfium::PdfiumBitmap;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::Key;
use winit::window::WindowId;

pub struct App {
    app_state: AppState,
    window_handler: WindowHandler,
    event_handler: EventHandler,
}

impl App {
    #[cfg(feature = "pdfium")]
    pub fn new(file: &str) -> Self {
        let mut app_state = AppState::new();
        let window_handler = WindowHandler::new();
        let mut event_handler = EventHandler::new(file);
        event_handler.init(&mut app_state);
        Self {
            app_state,
            window_handler,
            event_handler,
        }
    }

    #[cfg(not(feature = "pdfium"))]
    pub fn new(file: Vec<u8>) -> Self {
        let mut app_state = AppState::new();
        let window_handler = WindowHandler::new();
        let mut event_handler = EventHandler::new(file);
        event_handler.init(&mut app_state);
        Self {
            app_state,
            window_handler,
            event_handler,
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        self.app_state.max_page = self.app_state.pages.as_ref().unwrap().len() - 1;
        if self.app_state.pages.is_none() {
            println!("no pages to be displayed.");
        }
        self.window_handler.resumed(event_loop, &self.app_state);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                println!("ciao.");
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                self.draw();
            }
            WindowEvent::Resized(_) => {
                self.window_handler.request_redraw_wrapper();
            }
            WindowEvent::KeyboardInput {
                event: key_event, ..
            } => {
                // Only react to key presses, not releases.
                if key_event.state != winit::event::ElementState::Pressed {
                    return;
                }

                if let Key::Character(letter) = &key_event.logical_key {
                    self.event_handler
                        .handle(&mut self.app_state, letter.to_string());
                    self.window_handler.request_redraw_wrapper();
                    self.app_state.changed = false;
                }
            }
            _ => {}
        }
    }
}

impl App {
    fn draw(&mut self) {
        self.window_handler.draw(&mut self.app_state);
    }
}
