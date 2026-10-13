#[cfg(feature = "pdfium")]
use pdfium::PdfiumBitmap;

pub struct AppState<'a> {
    pub page: usize,
    pub max_page: usize,
    pub latest_scale: f32,
    pub scale: f32,
    pub vertical_scroll_pos: u32,
    pub horizontal_scroll_pos: u32,
    pub max_v_scroll: u32,
    pub max_h_scroll: u32,
    pub changed: bool,
    pub fits_vertically: bool,
    pub fits_horizontally: bool,
    #[cfg(feature = "pdfium")]
    pub pages: Option<&'a [PdfiumBitmap]>,
    #[cfg(not(feature = "pdfium"))]
    pub pages: Option<&'a [hayro::vello_cpu::Pixmap]>
}

impl AppState<'_> {
    pub fn new() -> Self {
        Self {
            page: 0,
            max_page: 0,
            latest_scale: 0.0,
            scale: 0.0,
            vertical_scroll_pos: 0,
            horizontal_scroll_pos: 0,
            max_v_scroll: 0,
            max_h_scroll: 0,
            changed: false,
            fits_vertically: false,
            fits_horizontally: false,
            #[cfg(feature = "pdfium")]
            pages: None,
            #[cfg(not(feature = "pdfium"))]
            pages: None
        }
    }
}
