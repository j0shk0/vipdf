use crate::state::AppState;
use rc::Rc;
use softbuffer::Surface;
use std::num::NonZeroU32;
use std::rc;
use winit::event_loop::ActiveEventLoop;
use winit::window::Window;

pub struct WindowHandler {
    window: Option<Rc<Window>>,
    surface: Option<Surface<Rc<Window>, Rc<Window>>>,
}

impl WindowHandler {

    pub fn new() -> Self {
        Self {
            window: None,
            surface: None,
        }
    }
    
    pub fn request_redraw_wrapper(&mut self) {
        if let Some(ref window) = self.window {
            window.request_redraw();
        }
    }

    pub fn resumed(&mut self, event_loop: &ActiveEventLoop, app_state: &AppState) {
        let window = Rc::new(
            event_loop
                .create_window(Window::default_attributes().with_title("vipdf"))
                .expect(
                    "create_window: needs a display server and a window config \
                in line with default window config.",
                ),
        );

        // Size the window to the first page (if there is one).
        let Some(ref pages) = app_state.pages else {
            println!("No pages to display");
            return;
        };
        if let Some(page) = pages.get(app_state.page) {
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


    pub fn draw(&mut self, app_state: &mut AppState) {
        let (Some(window), Some(surface)) = (self.window.as_ref(), self.surface.as_mut()) else {
            return;
        };

        let win_size = window.inner_size();
        app_state.window_height = win_size.height;
        app_state.refresh();

        let (Some(win_width), Some(win_height)) = (
            NonZeroU32::new(win_size.width),
            NonZeroU32::new(win_size.height),
        ) else {
            return;
        };

        surface
            .resize(win_width, win_height)
            .expect("surface.resize: window size rejected (out of range)");

        // Every State requires pixels. If that is not the case, we fail fast and loud.
        let page_pixels = app_state
            .pages
            .as_ref()
            .unwrap()
            .get(app_state.page)
            .unwrap();

        let page_width = page_pixels.width() as u32;
        let page_height = page_pixels.height() as u32;

        // Determine how much padding and how much actual page we have.
        let horizontal_padding = win_size.width.saturating_sub(page_width) / 2;
        let next_frame_page_width = page_width.min(win_size.width);

        // Clamping the vertical scroll position. No scrolling past (page_height - window_height).
        // If the page fits the window, scroll_position becomes 0.
        app_state.vertical_scroll_pos = app_state
            .vertical_scroll_pos
            .min(page_height.saturating_sub(win_size.height));

        // If the page is taller than the window, scroll within the page.
        // If it fits, no scrolling, only switching to the next or previous page.
        app_state.fits_vertically = page_height <= win_size.height;
        let vertical_padding = if app_state.fits_vertically {
            (win_size.height - page_height) / 2
        } else {
            0
        };

        // How many rows we can actually show: limited by window height and by
        // how many source rows remain below src_y0.
        let next_frame_page_height = page_height
            .saturating_sub(app_state.vertical_scroll_pos)
            .min(win_size.height.saturating_sub(vertical_padding));

        // Get buffer for the next frame.
        let mut buffer = surface.buffer_mut().expect(
            "surface.buffer_mut: surface was resized on the \
            line above, so it is configured",
        );
        buffer.fill(0x0000_0000);

        #[allow(unused)]
        let unwrapped_page_pixels: Vec<u8>;

        #[cfg(not(feature = "pdfium"))]
        let unwrapped_page_pixels = page_pixels.data();
        #[cfg(feature = "pdfium")]
        {
            let rgba_bytes = page_pixels.as_rgba_bytes();
            match rgba_bytes {
                Ok(bytes) => {
                    unwrapped_page_pixels = bytes;
                }
                Err(error) => {
                    panic!(
                        "page {} could not be converted to RGBA: {error}",
                        app_state.page
                    );
                }
            }
        }

        // Draw the page in the buffer.
        for y in 0..next_frame_page_height {
            let current_row = app_state.vertical_scroll_pos + y;
            for x in 0..next_frame_page_width {
                let horizontal_draw_pos = horizontal_padding + x;
                let vertical_draw_pos = vertical_padding + y;
                let draw_index =
                    (vertical_draw_pos * win_size.width + horizontal_draw_pos) as usize;

                #[cfg(not(feature = "pdfium"))]
                {
                    let page_pixel =
                        unwrapped_page_pixels[((current_row * page_width) + x) as usize];
                    let r = page_pixel.r as u32;
                    let g = page_pixel.g as u32;
                    let b = page_pixel.b as u32;
                    buffer[draw_index] = (r << 16) | (g << 8) | b;
                }

                #[cfg(feature = "pdfium")]
                {
                    let i = (((current_row * page_width) + x) * 4) as usize;
                    let r = unwrapped_page_pixels[i] as u32;
                    let g = unwrapped_page_pixels[i + 1] as u32;
                    let b = unwrapped_page_pixels[i + 2] as u32;
                    buffer[draw_index] = (r << 16) | (g << 8) | b;
                }
            }
        }
        buffer.present().unwrap();
    }
}
