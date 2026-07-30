use std::num::NonZeroU32;
use std::rc::Rc;
use softbuffer::Surface;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::Key;
use winit::window::{Window, WindowId};
use crate::parser::{Command, KeyParser};
use crate::renderer::Renderer;

const ZOOM_FACTOR: f32 = 0.3;
const SCROLL_STEP: u32 = 40;

pub struct App {
    window: Option<Rc<Window>>,
    surface: Option<Surface<Rc<Window>, Rc<Window>>>,
    pages: Vec<hayro::vello_cpu::Pixmap>,
    current_page: usize,
    latest_scale: f32,
    scale: f32,
    scroll_y: u32,
    parser: KeyParser,
    renderer: Renderer,
}

impl App {
    pub fn new(file: Vec<u8>) -> Self {
        let window = None;
        let surface: Option<Surface<Rc<Window>, Rc<Window>>> = None;
        let latest_scale = 1.0;
        let scale = latest_scale;
        let mut renderer = Renderer::new(file.clone());
        let pages = renderer.render_pdf(None, scale);
        let scroll_y = 0;
        let mut parser = KeyParser::default();
        parser.init();
        Self {
            window,
            latest_scale,
            scale,
            scroll_y,
            parser,
            renderer,
            surface,
            pages,
            current_page: 0,
        }
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
        let surface = Surface::new(&context, window.clone()).unwrap();

        self.surface = Some(surface);
        self.window = Some(window);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        let tmp_page_num = self.current_page;
        match event {
            WindowEvent::CloseRequested => {
                println!("The close button was pressed; stopping");
                event_loop.exit();
            }
            WindowEvent::RedrawRequested => {
                self.draw();
            }
            WindowEvent::Resized(_) => {
                if let Some(window) = self.window.as_ref() {
                    window.request_redraw();
                }
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
                    let command: Option<Command> = self.parser.read(letter.to_string());

                    match command {
                        None => {}
                        Some(Command::ScrollUp) => {
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
                        }
                        Some(Command::ScrollDown) => {
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
                        }
                        Some(Command::ZoomIn) => {
                            self.latest_scale += ZOOM_FACTOR;
                            self.scroll_y = 0; // scale change invalidates scroll position
                            changed = true;
                        }
                        Some(Command::ZoomOut) => {
                            if self.latest_scale - ZOOM_FACTOR >= 1.0 {
                                self.latest_scale -= ZOOM_FACTOR;
                                self.scroll_y = 0;
                                changed = true;
                            }
                        }
                        Some(Command::JumpToStart) => {
                            self.current_page = 0;
                            self.scroll_y = 0;
                            changed = true;
                        }
                        Some(Command::JumpToEnd) => {
                            self.current_page = self.pages.len() - 1;
                            let page_h = self.pages[self.current_page].height() as u32;
                            self.scroll_y = page_h.saturating_sub(win_h);
                            changed = true;
                        }
                    }

                    if changed {
                        if let Some(window) = self.window.as_ref() {
                            if self.current_page != tmp_page_num {
                                let pages = std::mem::take(&mut self.pages);
                                self.pages = self.renderer.render_pdf(
                                    Option::from((pages, self.current_page)),
                                    self.scale,
                                );
                            }
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

        if self.scale != self.latest_scale {
            let pages = std::mem::take(&mut self.pages);
            self.pages = self.renderer.render_pdf(
                Option::from((pages, self.current_page)),
                self.latest_scale,
            );
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
        // If it fits, center it and ignore the scroll.
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
