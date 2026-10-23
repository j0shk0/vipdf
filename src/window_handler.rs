use crate::state::AppState;
use rc::Rc;
use softbuffer::Surface;
use std::num::NonZeroU32;
use std::rc;
use winit::window::Window;

struct WindowHandler {
    window: Rc<Window>,
    surface: Surface<Rc<Window>, Rc<Window>>,
}

impl WindowHandler {
    fn draw(&mut self, next_state: &mut AppState) {
        let win_size = self.window.inner_size();
        let (Some(win_width), Some(win_height)) = (
            NonZeroU32::new(win_size.width),
            NonZeroU32::new(win_size.height),
        ) else {
            return;
        };

        self.surface
            .resize(win_width, win_height)
            .expect("surface.resize: window size rejected (out of range)");

        // Every State requires pixels. If that is not the case, we fail fast and loud.
        let page_pixels = next_state
            .pages
            .as_ref()
            .unwrap()
            .get(next_state.page)
            .unwrap();

        let page_width = page_pixels.width() as u32;
        let page_height = page_pixels.height() as u32;

        // Determine how much padding and how much actual page we have.
        let horizontal_padding = win_size.width.saturating_sub(page_width) / 2;
        let next_frame_page_width = page_width.min(win_size.width);

        // Clamping the vertical scroll position. No scrolling past (page_height - window_height).
        // If the page fits the window, scroll_position becomes 0.
        next_state.vertical_scroll_pos = next_state
            .vertical_scroll_pos
            .min(page_height.saturating_sub(win_size.height));

        // If the page is taller than the window, scroll within the page.
        // If it fits, no scrolling, only switching to the next or previous page.
        next_state.fits_vertically = page_height <= win_size.height;
        let vertical_padding = if next_state.fits_vertically {
            (win_size.height - page_height) / 2
        } else {
            0
        };

        // How many rows we can actually show: limited by window height and by
        // how many source rows remain below src_y0.
        let next_frame_page_height = page_height
            .saturating_sub(next_state.vertical_scroll_pos)
            .min(win_size.height.saturating_sub(vertical_padding));

        // Get buffer for the next frame.
        let mut buffer = self.surface.buffer_mut().expect(
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
            let rgba_bytes = unwrapped_page_pixels.as_rgba_bytes();
            match rgba_bytes {
                Ok(bytes) => {
                    unwrapped_page_pixels = bytes;
                }
                Err(error) => {
                    panic!(
                        "page {} could not be converted to RGBA: {error}",
                        next_state.page
                    );
                }
            }
        }

        // Draw the page in the buffer.
        for y in 0..next_frame_page_height {
            let current_row = next_state.vertical_scroll_pos + y;
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
