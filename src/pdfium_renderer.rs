#[allow(unused)]
use crate::state::AppState;
#[cfg(feature = "pdfium")]
use pdfium::*;

#[cfg(feature = "pdfium")]
const REPO: &str = "https://github.com/j0shk0/vipdf";

#[cfg(feature = "pdfium")]
pub struct PdfiumRenderer {
    pdf: PdfiumDocument,
}

#[cfg(feature = "pdfium")]
impl PdfiumRenderer {
    pub fn new(filename: &str) -> PdfiumResult<Self> {
        Ok(PdfiumRenderer {
            pdf: PdfiumDocument::new_from_path(filename, None)?,
        })
    }

    pub fn render_pdf(&mut self, app_state: &mut AppState) {
        let real_height = 1000.0 * app_state.scale;
        let config = PdfiumRenderConfig::default().with_height(real_height as i32);
        let mut all_pages: Vec<PdfiumBitmap>;
        let mut scales: Vec<f32>;
        let page = self.pdf.page(app_state.page as i32);
        if app_state.pages.is_some() {
            match page {
                Ok(page) => {
                    let rendering_result= page.render(&config);
                    app_state.pages.as_mut().unwrap()[app_state.page] = rendering_result.expect("Hm. This page could not be rendered.");
                }
                _ => {
                    let page_number = app_state.page;
                    println!(
                        "Hm. Page {page_number} of your document could not be rendered. \
                            Please consider opening an issue at {REPO} if you PDF is readable by other PDF Readers."
                    );
                    std::process::exit(1);
                }
            }
        } else {
            // For initial construction we render only the first page and provide placeholder
            // for lazy loading.
            all_pages = Vec::new();
            scales = Vec::new();
            for _ in self.pdf.pages() {
                all_pages.push(PdfiumBitmap::empty(1, 1, PdfiumBitmapFormat::Bgra).unwrap());
                scales.push(0.0);
            }
            let first_page = self.pdf.page(app_state.page as i32);
            match first_page {
                Ok(page) => {
                    let rendered_page = page.render(&config).unwrap();
                    all_pages[app_state.page] = rendered_page;
                }
                _ => {
                    println!(
                        "Hm. Page 0 of your document could not be rendered. \
                            Please consider opening an issue at {REPO} if you PDF is readable by other PDF Readers."
                    );
                    std::process::exit(1);
                }
            }
            app_state.pages = Some(all_pages);
            scales[0] = 1.0;
            app_state.scales = Some(scales);
        }
    }
}
