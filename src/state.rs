#[cfg(feature = "pdfium")]
use pdfium::PdfiumBitmap;

pub struct AppState {
    pub page: usize,
    pub max_page: usize,
    pub scale: f32,
    pub vertical_scroll_pos: u32,
    pub max_v_scroll: u32,
    pub window_height: u32,
    pub changed: bool,
    pub fits_vertically: bool,
    pub scales: Option<Vec<f32>>,
    #[cfg(feature = "pdfium")]
    pub pages: Option<Vec<PdfiumBitmap>>,
    #[cfg(not(feature = "pdfium"))]
    pub pages: Option<Vec<hayro::vello_cpu::Pixmap>>
}

impl AppState {
    pub fn new() -> Self {
        Self {
            page: 0,
            max_page: 0,
            scale: 1.0,
            vertical_scroll_pos: 0,
            max_v_scroll: 0,
            window_height: 0,
            changed: false,
            fits_vertically: true,

            scales: None,
            pages: None
        }
    }

    pub fn refresh(&mut self) {
        let page_height = self
            .pages
            .as_ref()
            .and_then(|pages| pages.get(self.page))
            .map_or(0, |page| page.height() as u32);
        self.fits_vertically = page_height <= self.window_height;
        self.max_v_scroll = page_height.saturating_sub(self.window_height);
    }

}
