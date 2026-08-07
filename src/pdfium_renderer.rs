#[cfg(feature = "pdfium")]
use pdfium::*;

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

    pub fn render_pdf(
        &mut self,
        pages: Option<(Vec<PdfiumBitmap>, usize)>,
        scale: f32,
    ) -> Vec<PdfiumBitmap> {
        let real_height = 1000.0 * scale;
        let config = PdfiumRenderConfig::default().with_height(real_height as i32);
        let mut all_pages: Vec<PdfiumBitmap>;
        let mut index: usize = 0;
        match pages {
            Some(val) => {
                all_pages = val.0;
                index = val.1;
                let page = self.pdf.page(index as i32);
                match page {
                    Ok(page) => {
                        let rendered_page = page.render(&config).unwrap();
                        all_pages[val.1] = rendered_page;
                        all_pages
                    }
                    _ => {
                        let page_number = val.1;
                        println!(
                            "Hm. Page {page_number} of your document could not be rendered. \
                            Please consider opening an issue at {REPO} if you PDF is readable by other PDF Readers."
                        );
                        std::process::exit(1);
                    }
                }
            }
            None => {
                // For initial construction we render only the first page and provide placeholder
                // for lazy loading.
                all_pages = Vec::new();
                for _ in self.pdf.pages() {
                    all_pages.push(PdfiumBitmap::empty(1, 1, PdfiumBitmapFormat::Bgra).unwrap());
                }
                let first_page = self.pdf.page(index as i32);
                match first_page {
                    Ok(page) => {
                        let rendered_page = page.render(&config).unwrap();
                        all_pages[index] = rendered_page;
                    }
                    _ => {
                        println!(
                            "Hm. Page 0 of your document could not be rendered. \
                            Please consider opening an issue at {REPO} if you PDF is readable by other PDF Readers."
                        );
                        std::process::exit(1);
                    }
                }
                all_pages
            }
        }
    }
}
