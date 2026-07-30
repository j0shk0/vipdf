use hayro::hayro_interpret::InterpreterSettings;
use hayro::hayro_interpret::font::{FontData, FontQuery, StandardFont};
use hayro::hayro_interpret::hayro_cmap::CidFamily;
use hayro::hayro_syntax::Pdf;
use hayro::{RenderCache, RenderSettings, render};
use std::path::Path;
use std::sync::Arc;
use vello_cpu::color::palette::css::WHITE;

fn load_asset(name: &str) -> Option<(FontData, u32)> {
    let base = Path::new(env!("CARGO_MANIFEST_DIR")).join("../hayro-tests/assets");
    let path = base.join(name);
    let data = std::fs::read(&path).ok()?;
    Some((Arc::new(data), 0))
}

pub fn render_pdf(file: Vec<u8>, scale: f32) -> Vec<hayro::vello_cpu::Pixmap> {

    let pdf = Pdf::new(file).unwrap();

    let interpreter_settings = InterpreterSettings {
        font_resolver: Arc::new(move |query| match query {
            FontQuery::Standard(s) => {
                let name = pick_standard_font(s);
                load_asset(name).or_else(|| Some(s.get_font_data()))
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
                            let name = pick_standard_font(&f.pick_standard_font());
                            return load_asset(name)
                                .or_else(|| Some(f.pick_standard_font().get_font_data()));
                        }
                    };

                    if let Some(data) = load_asset(name) {
                        return Some(data);
                    }
                }

                let name = pick_standard_font(&f.pick_standard_font());
                load_asset(name).or_else(|| Some(f.pick_standard_font().get_font_data()))
            }
        }),
        ..Default::default()
    };

    let render_settings = RenderSettings {
        x_scale: scale,
        y_scale: scale,
        bg_color: WHITE,
        ..Default::default()
    };

    let cache = RenderCache::new();

    let mut output: Vec<hayro::vello_cpu::Pixmap> = Vec::new();
    for page in pdf.pages().iter() {
        let _ = &output.push(render(page, &cache, &interpreter_settings, &render_settings));
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
