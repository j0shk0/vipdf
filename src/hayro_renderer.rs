use hayro::hayro_interpret::InterpreterSettings;
use hayro::hayro_interpret::font::{FontData, FontQuery, StandardFont};
use hayro::hayro_interpret::hayro_cmap::CidFamily;
use hayro::hayro_syntax::Pdf;
use hayro::{RenderCache, RenderSettings, render};
use std::path::Path;
use std::sync::Arc;
use hayro::vello_cpu::Pixmap;
use vello_cpu::color::palette::css::WHITE;

pub struct HayroRenderer {
    pdf: Pdf,
    interpreter_settings: InterpreterSettings,
}

impl HayroRenderer {

    pub fn new(file: Vec<u8>) -> Self {
        let pdf = Pdf::new(file).unwrap();
        let interpreter_settings = InterpreterSettings {
            font_resolver: Arc::new(move |query| match query {
                FontQuery::Standard(s) => {
                    let name = Self::pick_standard_font(s);
                    Self::load_asset(name).or_else(|| Some(s.get_font_data()))
                }
                FontQuery::Fallback(f) => {
                    if let Some(cc) = &f.character_collection {
                        let name = match cc.family {
                            CidFamily::AdobeGB1 | CidFamily::AdobeCNS1 => {
                                if f.is_bold {
                                    "NotoSansCJKsc-Bold.otf"
                                } else {
                                    "NotoSansCJKsc-Regular.otf"
                                }
                            }
                            CidFamily::AdobeJapan1 => {
                                if f.is_bold {
                                    "NotoSansCJKjp-Bold.otf"
                                } else {
                                    "NotoSansCJKjp-Regular.otf"
                                }
                            }
                            CidFamily::AdobeKorea1 => {
                                if f.is_bold {
                                    "NotoSansCJKkr-Bold.otf"
                                } else {
                                    "NotoSansCJKkr-Regular.otf"
                                }
                            }
                            _ => {
                                let name = Self::pick_standard_font(&f.pick_standard_font());
                                return Self::load_asset(name)
                                    .or_else(|| Some(f.pick_standard_font().get_font_data()));
                            }
                        };

                        if let Some(data) = Self::load_asset(name) {
                            return Some(data);
                        }
                    }

                    let name = Self::pick_standard_font(&f.pick_standard_font());
                    Self::load_asset(name).or_else(|| Some(f.pick_standard_font().get_font_data()))
                }
            }),
            ..Default::default()
        };
        Self { pdf, interpreter_settings }
    }

    pub fn render_pdf(
        &mut self,
        pages: Option<(Vec<Pixmap>, usize)>,
        scale: f32,
    ) -> Vec<Pixmap> {

        let render_settings = RenderSettings {
            x_scale: scale,
            y_scale: scale,
            bg_color: WHITE,
            ..Default::default()
        };

        let mut output: Vec<Pixmap> = Vec::new();

        match pages {
            None => {
                let cache = &RenderCache::new();
                let first_page = render(
                    &self.pdf.pages()[0],
                    cache,
                    &self.interpreter_settings,
                    &render_settings,
                );
                for _  in self.pdf.pages().iter() {
                    let _ = &output.push(Pixmap::new(0,0));
                }
                output[0] = first_page;
                print!("{}", output.len().to_string());
            }
            Some((pages, page)) => {
                let cache = &RenderCache::new();
                let rendered_page = render(
                    &self.pdf.pages()[page],
                    cache,
                    &self.interpreter_settings,
                    &render_settings,
                );
                output = pages;
                output[page] = rendered_page;
            }
        }
        output
    }

    fn pick_standard_font(font: &StandardFont) -> &'static str {
        match font {
            StandardFont::Helvetica => "LiberationSans-Regular.ttf",
            StandardFont::HelveticaBold => "LiberationSans-Bold.ttf",
            StandardFont::HelveticaOblique => "LiberationSans-Italic.ttf",
            StandardFont::HelveticaBoldOblique => "LiberationSans-BoldItalic.ttf",
            StandardFont::Courier => "LiberationMono-Regular.ttf",
            StandardFont::CourierBold => "LiberationMono-Bold.ttf",
            StandardFont::CourierOblique => "LiberationMono-Italic.ttf",
            StandardFont::CourierBoldOblique => "LiberationMono-BoldItalic.ttf",
            StandardFont::TimesRoman => "LiberationSerif-Regular.ttf",
            StandardFont::TimesBold => "LiberationSerif-Bold.ttf",
            StandardFont::TimesItalic => "LiberationSerif-Italic.ttf",
            StandardFont::TimesBoldItalic => "LiberationSerif-BoldItalic.ttf",
            StandardFont::ZapfDingBats => "FoxitDingbats.pfb",
            StandardFont::Symbol => "FoxitSymbol.pfb",
        }
    }

    fn load_asset(name: &str) -> Option<(FontData, u32)> {
        let base = Path::new(env!("CARGO_MANIFEST_DIR")).join("../hayro-tests/assets");
        let path = base.join(name);
        let data = std::fs::read(&path).ok()?;
        Some((Arc::new(data), 0))
    }

}


