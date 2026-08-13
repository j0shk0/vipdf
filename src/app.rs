#[cfg(not(feature = "pdfium"))]
use crate::hayro_renderer::HayroRenderer;
use crate::parser::{Command, KeyParser};
#[cfg(feature = "pdfium")]
use crate::pdfium_renderer::PdfiumRenderer;
#[cfg(feature = "pdfium")]
use pdfium::PdfiumBitmap;
use softbuffer::Surface;
use std::num::NonZeroU32;
use std::rc::Rc;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::Key;
use winit::window::{Window, WindowId};

const ZOOM_FACTOR: f32 = 0.3;
const SCROLL_STEP: u32 = 40;

pub struct App {
    window: Option<Rc<Window>>,
    surface: Option<Surface<Rc<Window>, Rc<Window>>>,
    page: usize,
    latest_scale: f32,
    scale: f32,
    vertical_scroll_pos: u32,
    parser: KeyParser,

    #[cfg(feature = "pdfium")]
    pages: Vec<PdfiumBitmap>,
    #[cfg(not(feature = "pdfium"))]
    pages: Vec<hayro::vello_cpu::Pixmap>,

    #[cfg(feature = "pdfium")]
    renderer: PdfiumRenderer,
    #[cfg(not(feature = "pdfium"))]
    renderer: HayroRenderer,
}

impl App {
    #[cfg(feature = "pdfium")]
    pub fn new(file: &str) -> Self {
        let window = None;
        let surface: Option<Surface<Rc<Window>, Rc<Window>>> = None;
        let latest_scale = 1.0;
        let scale = latest_scale;
        let renderer_result = PdfiumRenderer::new(file);
        let mut renderer = match renderer_result {
            Ok(r) => r,
            Err(r) => {
                panic!(
                    "could not open PDF with pdfium (missing file, not a PDF, or encrypted): {r}"
                )
            }
        };
        let pages = renderer.render_pdf(None, 1.0);
        let mut parser = KeyParser::default();
        parser.init();
        Self {
            window,
            latest_scale,
            scale,
            vertical_scroll_pos: 0,
            parser,
            renderer,
            surface,
            pages,
            page: 0,
        }
    }

    #[cfg(not(feature = "pdfium"))]
    pub fn new(file: Vec<u8>) -> Self {
        let window = None;
        let surface: Option<Surface<Rc<Window>, Rc<Window>>> = None;
        let latest_scale = 1.0;
        let scale = latest_scale;
        let mut renderer = HayroRenderer::new(file.clone());

        let pages = renderer.render_pdf(None, scale);
        let mut parser = KeyParser::default();
        parser.init();
        Self {
            window,
            latest_scale,
            scale,
            vertical_scroll_pos: 0,
            parser,
            renderer,
            surface,
            pages,
            page: 0,
        }
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = Rc::new(
            event_loop
                .create_window(Window::default_attributes().with_title("vipdf"))
                .expect(
                    "create_window: needs a display server and a window config \
                in line with default window config.",
                ),
        );

        // Size the window to the first page (if there is one).
        if let Some(page) = self.pages.get(self.page) {
            let _ = window.request_inner_size(winit::dpi::PhysicalSize::new(
                page.width() as u32,
                page.height() as u32,
            ));
        }

        let context = softbuffer::Context::new(window.clone())
            .expect("softbuffer::Context::new: display platform not supported by softbuffer");
        let surface = Surface::new(&context, window.clone())
            .expect("softbuffer::Surface::new: window handle rejected. Are you on an unsupported platform ?");

        self.surface = Some(surface);
        self.window = Some(window);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _id: WindowId, event: WindowEvent) {
        let tmp_page_num = self.page;
        match event {
            WindowEvent::CloseRequested => {
                println!("ciao.");
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
                let mut already_rendered= false;

                if let Key::Character(letter) = &key_event.logical_key {
                    // Figure out whether the current page is taller than the window.
                    let win_h = self
                        .window
                        .as_ref()
                        .map(|w| w.inner_size().height)
                        .unwrap_or(0);
                    let page_h = self
                        .pages
                        .get(self.page)
                        .map(|p| p.height() as u32)
                        .unwrap_or(0);

                    // How far we can scroll before the bottom of the page reaches
                    // the bottom of the window. 0 if the page fits.
                    let max_scroll = page_h.saturating_sub(win_h);
                    let command: Option<Command> = self.parser.read(letter.to_string());

                    match command {
                        None => {}
                        Some(Command::ScrollDown) => {
                            if self.vertical_scroll_pos < max_scroll {
                                // Still room to scroll down within this page.
                                self.vertical_scroll_pos =
                                    (self.vertical_scroll_pos + SCROLL_STEP).min(max_scroll);
                                changed = true;
                            } else if self.page + 1 < self.pages.len() {
                                // Bottom reached -> next page.
                                self.page += 1;
                                self.vertical_scroll_pos = 0;
                                changed = true;
                            }
                        }
                        Some(Command::ScrollDownN(num)) => {
                            let prev_h = self.pages[self.page].height() as u32;
                            if self.vertical_scroll_pos < max_scroll {
                                // Still room to scroll down within this page.
                                self.vertical_scroll_pos = (self.vertical_scroll_pos
                                    + SCROLL_STEP * num as u32)
                                    .min(max_scroll);
                                changed = true;
                                // Discriminate by zoom level.
                            } else if prev_h <= win_h {
                                if self.page + num < self.pages.len() {
                                    self.page += num;
                                    self.vertical_scroll_pos = 0;
                                    changed = true;
                                } else {
                                    self.page = self.pages.len() - 1;
                                    changed = true;
                                }
                            } else if self.page + 1 < self.pages.len() {
                                // Bottom reached -> next page.
                                self.page += 1;
                                self.vertical_scroll_pos = 0;
                                changed = true;
                            }
                        }
                        Some(Command::ScrollUp) => {
                            if self.vertical_scroll_pos > 0 {
                                // Still room to scroll up within this page.
                                self.vertical_scroll_pos =
                                    self.vertical_scroll_pos.saturating_sub(SCROLL_STEP);
                                changed = true;
                            } else if self.page > 0 {
                                // Top reached -> previous page, land at its bottom.
                                self.page -= 1;

                                self.scale = self.latest_scale; // Avoiding rerender by draw function.
                                let pages = std::mem::take(&mut self.pages);
                                self.pages = self
                                    .renderer
                                    .render_pdf(Option::from((pages, self.page)), self.scale);
                                already_rendered = true;

                                let page_h = self.pages[self.page].height() as u32;
                                self.vertical_scroll_pos = page_h.saturating_sub(win_h);
                                changed = true;
                            }
                        }
                        Some(Command::ScrollUpN(num)) => {
                            let page_h = self.pages[self.page].height() as u32;
                            if self.vertical_scroll_pos > 0 {
                                // Still room to scroll up within this page.
                                self.vertical_scroll_pos = self
                                    .vertical_scroll_pos
                                    .saturating_sub(SCROLL_STEP * num as u32);
                                changed = true;
                                // Discriminate for zoom level.
                            } else if page_h <= win_h {
                                if self.page.saturating_sub(num) > 0 {
                                    self.page = self.page.saturating_sub(num);
                                    self.vertical_scroll_pos = 0;
                                    changed = true;
                                } else {
                                    self.page = 0;
                                    changed = true;
                                }
                            } else if self.page > 0 {
                                // Top reached -> previous page, land at its bottom.
                                self.page -= 1;

                                self.scale = self.latest_scale; // Avoiding rerender by draw function.
                                let pages = std::mem::take(&mut self.pages);
                                self.pages = self
                                    .renderer
                                    .render_pdf(Option::from((pages, self.page)), self.scale);
                                already_rendered = true;

                                let page_h = self.pages[self.page].height() as u32;
                                self.vertical_scroll_pos = page_h.saturating_sub(win_h);
                                changed = true;
                            }
                        }
                        Some(Command::ZoomIn) => {
                            self.latest_scale += ZOOM_FACTOR;
                            self.vertical_scroll_pos = 0; // scale change invalidates scroll position
                            changed = true;
                        }
                        Some(Command::ZoomOut) => {
                            if self.latest_scale - ZOOM_FACTOR >= 0.01 {
                                self.latest_scale -= ZOOM_FACTOR;
                                self.vertical_scroll_pos = 0;
                                changed = true;
                            }
                        }
                        Some(Command::JumpToStart) => {
                            self.page = 0;
                            self.vertical_scroll_pos = 0;
                            changed = true;
                        }
                        Some(Command::JumpToEnd) => {
                            self.page = self.pages.len() - 1;

                            self.scale = self.latest_scale; // Avoiding rerender by draw function.
                            let pages = std::mem::take(&mut self.pages);
                            self.pages = self
                                .renderer
                                .render_pdf(Option::from((pages, self.page)), self.scale);
                            already_rendered = true;

                            let page_h = self.pages[self.page].height() as u32;
                            self.vertical_scroll_pos = page_h.saturating_sub(win_h);
                            changed = true;
                        }
                        Some(Command::JumpToPage(num)) => {
                            self.page = num.clamp(0, self.pages.len() - 1);
                            self.vertical_scroll_pos = 0;
                            changed = true;
                        }
                    }

                    if changed {
                        if let Some(window) = self.window.as_ref() {
                            if self.page != tmp_page_num && !already_rendered {
                                let pages = std::mem::take(&mut self.pages);
                                self.scale = self.latest_scale;
                                self.pages = self
                                    .renderer
                                    .render_pdf(Option::from((pages, self.page)), self.scale);
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
            self.pages = self
                .renderer
                .render_pdf(Option::from((pages, self.page)), self.latest_scale);
            self.scale = self.latest_scale
        }

        let Some(page) = self.pages.get(self.page) else {
            return;
        };

        let win_size = window.inner_size();
        let (Some(win_width), Some(win_height)) = (
            NonZeroU32::new(win_size.width),
            NonZeroU32::new(win_size.height),
        ) else {
            return;
        };

        surface
            .resize(win_width, win_height)
            .expect("surface.resize: window size rejected (out of range)");

        let page_width = page.width() as u32;
        let page_height = page.height() as u32;

        let horizontal_padding = win_size.width.saturating_sub(page_width) / 2;

        // Clamping scroll_y. No scrolling past (page_height - window_height).
        // If the page fits the window, scroll_position becomes 0.
        self.vertical_scroll_pos = self
            .vertical_scroll_pos
            .min(page_height.saturating_sub(win_size.height));

        // If the page is taller than the window, scroll within the page.
        // If it fits, no scrolling, only switching to the next or previous page.
        let fits_vertically = page_height <= win_size.height;
        let vertical_padding = if fits_vertically {
            (win_size.height - page_height) / 2
        } else {
            0
        };

        // Get buffer for the next frame.
        let mut buffer = surface.buffer_mut().expect(
            "surface.buffer_mut: surface was resized on the \
            line above, so it is configured",
        );
        buffer.fill(0x0000_0000);

        let next_frame_page_width = page_width.min(win_size.width);

        // How many rows we can actually show: limited by window height and by
        // how many source rows remain below src_y0.
        let next_frame_page_height = page_height
            .saturating_sub(self.vertical_scroll_pos)
            .min(win_size.height.saturating_sub(vertical_padding));

        #[allow(unused)]
        let page_pixels: Vec<u8>;

        #[cfg(not(feature = "pdfium"))]
        let page_pixels = page.data();
        #[cfg(feature = "pdfium")]
        {
            let rgba_bytes = page.as_rgba_bytes();
            match rgba_bytes {
                Ok(bytes) => {
                    page_pixels = bytes;
                }
                Err(error) => {
                    panic!("page {} could not be converted to RGBA: {error}", self.page);
                }
            }
        }

        // Draw the page in the buffer.
        for y in 0..next_frame_page_height {
            let current_row = self.vertical_scroll_pos + y;
            for x in 0..next_frame_page_width {
                let horizontal_draw_pos = horizontal_padding + x;
                let vertical_draw_pos = vertical_padding + y;
                let draw_index =
                    (vertical_draw_pos * win_size.width + horizontal_draw_pos) as usize;

                #[cfg(not(feature = "pdfium"))]
                {
                    let page_pixel = page_pixels[((current_row * page_width) + x) as usize];
                    let r = page_pixel.r as u32;
                    let g = page_pixel.g as u32;
                    let b = page_pixel.b as u32;
                    buffer[draw_index] = (r << 16) | (g << 8) | b;
                }

                #[cfg(feature = "pdfium")]
                {
                    let i = (((current_row * page_width) + x) * 4) as usize;
                    let r = page_pixels[i] as u32;
                    let g = page_pixels[i + 1] as u32;
                    let b = page_pixels[i + 2] as u32;
                    buffer[draw_index] = (r << 16) | (g << 8) | b;
                }
            }
        }
        buffer.present().unwrap();
    }
}
