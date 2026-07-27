mod render;

use crate::render::render_pdf;
use std::num::NonZeroU32;
use std::rc::Rc;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::keyboard::Key;
use winit::window::{Window, WindowId};

const ZOOM_FACTOR: f32 = 0.3;
const SCROLL_STEP: u32 = 40;

#[derive(Default)]
struct App {
    window: Option<Rc<Window>>,
    surface: Option<softbuffer::Surface<Rc<Window>, Rc<Window>>>,
    file: Vec<u8>,
    pages: Vec<hayro::vello_cpu::Pixmap>,
    current_page: usize,
    latest_scale: f32,
    scale: f32,
    scroll_y: u32, // Add scroll_y field
}

impl App {
    fn init(&mut self, file: Vec<u8>) {
        self.file = file;
        self.latest_scale = 1.0;
        self.scale = self.latest_scale;
        self.pages = render_pdf(self.file.clone(), self.scale);
        self.scroll_y = 0; // Initialize scroll_y
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = Rc::new(
            event_loop
                .create_window(Window::default_attributes().with_title("vipdf"))
                .unwrap(),
        );

        // Size the window to the first page (if there is one).
        if let Some(page) = self.pages.get(self.current_page) {
            let _ = window.request_inner_size(winit::dpi::PhysicalSize::new(
                page.width() as u32,
                page.height() as u32,
            ));
        }

        let context = softbuffer::Context::new(window.clone()).unwrap();
        let surface = softbuffer::Surface::new(&context, window.clone()).unwrap();

        self.surface = Some(surface);
        self.window = Some(window);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        match event {
            WindowEvent::CloseRequested => {
                println!("The close button was pressed; stopping");
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                self.draw();
            }
            WindowEvent::KeyboardInput {
                event: key_event, ..
            } => {
                // Only react to key presses, not releases.
                if key_event.state != winit::event::ElementState::Pressed {
                    return;
                }

                let mut changed = false;

                if let Key::Character(letter) = &key_event.logical_key {
                    // Figure out whether the current page is taller than the window.
                    let win_h = self
                        .window
                        .as_ref()
                        .map(|w| w.inner_size().height)
                        .unwrap_or(0);
                    let page_h = self
                        .pages
                        .get(self.current_page)
                        .map(|p| p.height() as u32)
                        .unwrap_or(0);

                    // How far we can scroll before the bottom of the page reaches
                    // the bottom of the window. 0 if the page fits.
                    let max_scroll = page_h.saturating_sub(win_h);

                    if letter == "j" {
                        if self.scroll_y < max_scroll {
                            // Still room to scroll down within this page.
                            self.scroll_y = (self.scroll_y + SCROLL_STEP).min(max_scroll);
                            changed = true;
                        } else if self.current_page + 1 < self.pages.len() {
                            // Bottom reached -> next page.
                            self.current_page += 1;
                            self.scroll_y = 0;
                            changed = true;
                        }
                    } else if letter == "k" {
                        if self.scroll_y > 0 {
                            // Still room to scroll up within this page.
                            self.scroll_y = self.scroll_y.saturating_sub(SCROLL_STEP);
                            changed = true;
                        } else if self.current_page > 0 {
                            // Top reached -> previous page, land at its bottom.
                            self.current_page -= 1;
                            let prev_h = self.pages[self.current_page].height() as u32;
                            self.scroll_y = prev_h.saturating_sub(win_h);
                            changed = true;
                        }
                    } else if letter == "+" {
                        self.latest_scale += ZOOM_FACTOR;
                        self.scroll_y = 0; // scale change invalidates scroll position
                        changed = true;
                    } else if letter == "-" {
                        if self.latest_scale - ZOOM_FACTOR >= 1.0 {
                            self.latest_scale -= ZOOM_FACTOR;
                            self.scroll_y = 0;
                            changed = true;
                        }
                    }

                    if changed {
                        if let Some(window) = self.window.as_ref() {
                            window.request_redraw();
                        }
                    }
                }
            }
            _ => (),
        }
    }
}

impl App {
    fn draw(&mut self) {
        let (Some(window), Some(surface)) = (self.window.as_ref(), self.surface.as_mut()) else {
            return;
        };

        if (self.scale != self.latest_scale) {
            self.pages = render_pdf(self.file.clone(), self.latest_scale);
            self.scale = self.latest_scale
        }

        let Some(page) = self.pages.get(self.current_page) else {
            return;
        };

        let size = window.inner_size();
        let (Some(win_width), Some(win_height)) =
            (NonZeroU32::new(size.width), NonZeroU32::new(size.height))
        else {
            return;
        };

        surface.resize(win_width, win_height).unwrap();

        let pix_w = page.width() as u32;
        let pix_h = page.height() as u32;

        // Horizontal: center as before.
        let offset_x = size.width.saturating_sub(pix_w) / 2;

        // Vertical: if the page is taller than the window, top-align and scroll.
        // If it fits, center it and ignore scroll.
        let fits_vertically = pix_h <= size.height;
        if !fits_vertically {
            self.scroll_y = self.scroll_y.min(pix_h.saturating_sub(size.height));
        }
        let offset_y = if fits_vertically {
            size.height.saturating_sub(pix_h) / 2
        } else {
            0
        };
        let src_y0 = if fits_vertically { 0 } else { self.scroll_y };

        let mut buffer = surface.buffer_mut().unwrap();
        buffer.fill(0x0000_0000);

        let copy_w = pix_w.min(size.width);
        // How many rows we can actually show: limited by window height and by
        // how many source rows remain below src_y0.
        let copy_h = pix_h
            .saturating_sub(src_y0)
            .min(size.height.saturating_sub(offset_y));

        let pixels = page.data();

        for y in 0..copy_h {
            let src_row = src_y0 + y; // shifted by scroll
            for x in 0..copy_w {
                let px = pixels[((src_row * pix_w) + x) as usize];

                let r = px.r as u32;
                let g = px.g as u32;
                let b = px.b as u32;

                let dst_x = offset_x + x;
                let dst_y = offset_y + y;
                let dst = (dst_y * size.width + dst_x) as usize;

                buffer[dst] = (r << 16) | (g << 8) | b;
            }
        }

        buffer.present().unwrap();
    }
}

fn main() {
    let file = std::fs::read(std::env::args().nth(1).unwrap()).unwrap();

    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Wait);

    let mut app = App::default();
    app.init(file);
    event_loop.run_app(&mut app).unwrap();
}
