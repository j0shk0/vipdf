use crate::parser::{Command, KeyParser};
use crate::state::AppState;

const ZOOM_FACTOR: f32 = 0.3;
const SCROLL_V_STEP: u32 = 40;

pub struct EventHandler {
    parser: KeyParser,
}

impl EventHandler {
    #[cfg(feature = "pdfium")]
    pub fn new(file: &str) -> Self {
        let mut parser = KeyParser::default();
        parser.init();
        Self { parser }
    }

    #[cfg(not(feature = "pdfium"))]
    pub fn new(file: Vec<u8>) -> Self {
        let mut parser = KeyParser::default();
        parser.init();
        Self { parser }
    }

    // TODO: Think about how to avoid copy of input_state
    fn handle(&mut self, input_state: &mut AppState, input: String) {
        let maybe_command = self.parser.read(input);

        match maybe_command {
            None => {},
            Some(Command::ScrollDown) => {
                if input_state.vertical_scroll_pos < input_state.max_v_scroll {
                    // Still room to scroll down within this page.
                    input_state.vertical_scroll_pos = (input_state.vertical_scroll_pos
                        + SCROLL_V_STEP)
                        .min(input_state.max_v_scroll);
                    input_state.changed = true;
                } else if input_state.page < input_state.max_page {
                    // Bottom reached -> next page.
                    input_state.page += 1;
                    input_state.vertical_scroll_pos = 0;
                    input_state.changed = true;
                }
            }
            Some(Command::ScrollDownN(num)) => {
                // Is the page fully displayed?
                // If true, we are not scrolling inside a page but across pages.
                if input_state.fits_vertically {
                    if input_state.page + num - 1 < input_state.max_page {
                        // Would we go out of bounds?
                        input_state.page += num;
                        input_state.vertical_scroll_pos = 0;
                        input_state.changed = true;
                    } else {
                        // We would go out of bounds and advance directly to the last page.
                        input_state.page = input_state.max_page;
                        input_state.changed = true;
                    }
                } else if input_state.vertical_scroll_pos < input_state.max_v_scroll {
                    // Page is not fully displayed.
                    // Is there still room to scroll within this page?
                    input_state.vertical_scroll_pos = (input_state.vertical_scroll_pos
                        + SCROLL_V_STEP * num as u32)
                        .min(input_state.max_v_scroll);
                    input_state.changed = true;
                } else if input_state.page < input_state.max_page {
                    // No room to scroll go to top of next page.
                    input_state.page += 1;
                    input_state.vertical_scroll_pos = 0;
                    input_state.changed = true;
                }
            }
            Some(Command::ScrollUp) => {
                // Can we still scroll up within this page?
                if input_state.vertical_scroll_pos > 0 {
                    input_state.vertical_scroll_pos = input_state
                        .vertical_scroll_pos
                        .saturating_sub(SCROLL_V_STEP);
                    input_state.changed = true;
                } else if input_state.page > 0 {
                    input_state.page -= 1;
                    println!("CAREFUL: THE COMMAND SCROLL UP IS NOT FUNCTIONAL RIGHT NOW.");
                    // TODO: Decide where this should be handled.
                    // input_state.goto_page_safe(&mut input_state.changed, &mut already_rendered, win_h);
                }
            }
            Some(Command::ScrollUpN(num)) => {
                // is the page fully displayed?
                // if true, we are not scrolling inside a page but across pages.
                if input_state.fits_vertically {
                    if input_state.page.saturating_sub(num) > 0 {
                        // Would we go out of bounds?
                        input_state.page = input_state.page.saturating_sub(num);
                        input_state.vertical_scroll_pos = 0;
                        input_state.changed = true;
                    } else {
                        // We would go out of bounds and advance to the first page.
                        input_state.page = 0;
                        input_state.changed = true;
                    }
                } else if input_state.vertical_scroll_pos > 0 {
                    // Can we still scroll up within this page?
                    input_state.vertical_scroll_pos = input_state
                        .vertical_scroll_pos
                        .saturating_sub(SCROLL_V_STEP * num as u32);
                    input_state.changed = true;
                } else if input_state.page > 0 {
                    input_state.page -= 1;
                    println!("CAREFUL: THE COMMAND SCROLL UP N IS NOT FUNCTIONAL RIGHT NOW.");
                    // TODO: Decide where this should be handled.
                    // input_state.goto_page_safe(&mut input_state.changed, &mut already_rendered, win_h);
                }
            }
            Some(Command::ZoomIn) => {
                input_state.latest_scale += ZOOM_FACTOR;
                input_state.vertical_scroll_pos = 0; // scale change invalidates scroll position
                input_state.changed = true;
            }
            Some(Command::ZoomOut) => {
                if input_state.latest_scale - ZOOM_FACTOR >= 0.01 {
                    input_state.latest_scale -= ZOOM_FACTOR;
                    input_state.vertical_scroll_pos = 0;
                    input_state.changed = true;
                }
            }
            Some(Command::JumpToStart) => {
                input_state.page = 0;
                input_state.vertical_scroll_pos = 0;
                input_state.changed = true;
            }
            Some(Command::JumpToEnd) => {
                input_state.page = input_state.max_page;
                println!("CAREFUL: THE COMMAND SCROLL UP N IS NOT FUNCTIONAL RIGHT NOW.");
                // TODO: Decide where this should be handled.
                // input_state.goto_page_safe(&mut input_state.changed, &mut already_rendered, win_h);
            }
            Some(Command::JumpToPage(num)) => {
                input_state.page = num.clamp(0, input_state.max_page);
                input_state.vertical_scroll_pos = 0;
                input_state.changed = true;
            }
        }
    }
}
