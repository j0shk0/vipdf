#[cfg(not(feature = "pdfium"))]
use crate::hayro_renderer::HayroRenderer;
use crate::parser::{Command, KeyParser};
#[cfg(feature = "pdfium")]
use crate::pdfium_renderer::PdfiumRenderer;
use crate::state::AppState;

pub struct EventHandler {
    parser: KeyParser,
    pub zoom_factor: f32,
    pub scroll_v_step: u32,

    #[cfg(feature = "pdfium")]
    renderer: PdfiumRenderer,
    #[cfg(not(feature = "pdfium"))]
    renderer: HayroRenderer,
}

impl EventHandler {
    #[cfg(feature = "pdfium")]
    pub fn new(file: &str) -> Self {
        let mut parser = KeyParser::default();
        let zoom_factor: f32 = 0.3;
        let scroll_v_step: u32 = 40;
        parser.init();
        let renderer_result = PdfiumRenderer::new(file);
        let renderer = match renderer_result {
            Ok(r) => r,
            Err(r) => {
                panic!(
                    "could not open PDF with pdfium (missing file, not a PDF, or encrypted): {r}"
                )
            }
        };
        Self {
            parser,
            zoom_factor,
            scroll_v_step,
            renderer,
        }
    }

    #[cfg(not(feature = "pdfium"))]
    pub fn new(file: Vec<u8>) -> Self {
        let mut parser = KeyParser::default();
        parser.init();
        let renderer = HayroRenderer::new(file.clone());
        let zoom_factor: f32 = 0.3;
        let scroll_v_step: u32 = 40;
        Self {
            parser,
            zoom_factor,
            scroll_v_step,
            renderer,
        }
    }

    pub fn init(&mut self, app_state: &mut AppState) {
        self.renderer.render_pdf(app_state);
    }

    pub fn handle(&mut self, input_state: &mut AppState, input: String) {
        let maybe_command = self.parser.read(input);

        match maybe_command {
            None => {}

            Some(Command::ScrollDown) => {
                // Are we zoomed out ?
                if input_state.fits_vertically {
                    // Already on the last page ?
                    if input_state.page < input_state.max_page {
                        input_state.page += 1;
                        self.maybe_rerender(input_state);
                        input_state.vertical_scroll_pos = 0;
                        input_state.changed = true;
                    }
                    // Are we already at the bottom of the page ?
                } else if input_state.vertical_scroll_pos < input_state.max_v_scroll {
                    input_state.vertical_scroll_pos = (input_state.vertical_scroll_pos
                        + self.scroll_v_step)
                        .min(input_state.max_v_scroll);
                    input_state.changed = true;
                } else if input_state.page < input_state.max_page {
                    // Next page exists?
                    input_state.page += 1;
                    self.maybe_rerender(input_state);
                    input_state.vertical_scroll_pos = 0;
                    input_state.changed = true;
                }
            }

            Some(Command::ScrollDownN(num)) => {
                // Are we zoomed out ?
                if input_state.fits_vertically {
                    // Would we go beyond last page ?
                    if input_state.page + num - 1 < input_state.max_page {
                        input_state.page += num;
                        self.maybe_rerender(input_state);
                        input_state.changed = true;
                    } else if input_state.page < input_state.max_page {
                        // Already on last page?
                        input_state.page = input_state.max_page;
                        self.maybe_rerender(input_state);
                        input_state.vertical_scroll_pos = 0;
                        input_state.changed = true;
                    }
                } else if input_state.vertical_scroll_pos < input_state.max_v_scroll {
                    // Page is not fully displayed.
                    // Is there still room to scroll within this page?
                    input_state.vertical_scroll_pos = (input_state.vertical_scroll_pos
                        + self.scroll_v_step * num as u32)
                        .min(input_state.max_v_scroll);
                    input_state.changed = true;
                } else if input_state.page < input_state.max_page {
                    // No room to scroll go to top of next page.
                    input_state.page += 1;
                    self.maybe_rerender(input_state);
                    input_state.vertical_scroll_pos = 0;
                    input_state.changed = true;
                }
            }

            Some(Command::ScrollUp) => {
                // Can we still scroll up within this page?
                if input_state.fits_vertically {
                    if input_state.page > 0 {
                        input_state.page -= 1;
                        self.maybe_rerender(input_state);
                        input_state.vertical_scroll_pos = input_state.max_v_scroll;
                        input_state.changed = true;
                    }
                } else {
                    if input_state.vertical_scroll_pos > 0 {
                        input_state.vertical_scroll_pos = input_state
                            .vertical_scroll_pos
                            .saturating_sub(self.scroll_v_step);
                        input_state.changed = true;
                    } else {
                        let old_page = input_state.page;
                        input_state.page = input_state.page.saturating_sub(1);
                        self.maybe_rerender(input_state);
                        if old_page != input_state.page {
                            input_state.vertical_scroll_pos = input_state.max_v_scroll;
                            input_state.changed = true;
                        }
                    }
                }
            }

            Some(Command::ScrollUpN(num)) => {
                // Can we still scroll up within this page?
                if input_state.fits_vertically {
                    if input_state.page > 0 {
                        let old_page = input_state.page;
                        input_state.page = input_state.page.saturating_sub(num);
                        self.maybe_rerender(input_state);
                        if old_page != input_state.page {
                            input_state.vertical_scroll_pos = input_state.max_v_scroll;
                            input_state.changed = true;
                        }
                    }
                } else {
                    if input_state.vertical_scroll_pos > 0 {
                        input_state.vertical_scroll_pos = input_state
                            .vertical_scroll_pos
                            .saturating_sub(self.scroll_v_step * num as u32);
                        input_state.changed = true;
                    } else {
                        let old_page = input_state.page;
                        input_state.page = input_state.page.saturating_sub(num);
                        self.maybe_rerender(input_state);
                        if old_page != input_state.page {
                            input_state.vertical_scroll_pos = input_state.max_v_scroll;
                            input_state.changed = true;
                        }
                    }
                }
            }

            Some(Command::ZoomIn) => {
                input_state.scale += self.zoom_factor;
                self.maybe_rerender(input_state);
                input_state.vertical_scroll_pos = 0;
                input_state.changed = true;
            }

            Some(Command::ZoomOut) => {
                if input_state.scale - self.zoom_factor >= 0.01 {
                    input_state.scale -= self.zoom_factor;
                    self.maybe_rerender(input_state);
                    input_state.vertical_scroll_pos = 0;
                    input_state.changed = true;
                }
            }

            Some(Command::JumpToStart) => {
                input_state.page = 0;
                self.maybe_rerender(input_state);
                input_state.vertical_scroll_pos = 0;
                input_state.changed = true;
            }

            Some(Command::JumpToEnd) => {
                input_state.page = input_state.max_page;
                self.maybe_rerender(input_state);
                input_state.vertical_scroll_pos = input_state.max_v_scroll;
                input_state.changed = true;
            }

            Some(Command::JumpToPage(num)) => {
                input_state.page = num.clamp(0, input_state.max_page);
                self.maybe_rerender(input_state);
                input_state.vertical_scroll_pos = 0;
                input_state.changed = true;
            }
        }
    }

    fn maybe_rerender(&mut self, input_state: &mut AppState) {
        let page_scale = input_state
            .scales
            .as_ref()
            .unwrap()
            .get(input_state.page)
            .unwrap();
        if *page_scale != input_state.scale {
            self.renderer.render_pdf(input_state);
            input_state.scales.as_mut().unwrap()[input_state.page] = input_state.scale;
        }

        input_state.refresh();
    }
}
