use crate::state::AppState;
use hayro::hayro_interpret::InterpreterSettings;
use hayro::hayro_interpret::font::FontQuery;
use hayro::hayro_syntax::Pdf;
use hayro::vello_cpu::Pixmap;
use hayro::{RenderCache, RenderSettings, render};
use std::sync::Arc;
use vello_cpu::color::palette::css::WHITE;

pub struct HayroRenderer {
    pdf: Pdf,
    interpreter_settings: InterpreterSettings,
}

#[allow(dead_code)]
impl HayroRenderer {
    #[allow(dead_code)]
    pub fn new(file: Vec<u8>) -> Self {
        let pdf = Pdf::new(file).unwrap();
        let interpreter_settings = InterpreterSettings {
            font_resolver: Arc::new(|query| match query {
                FontQuery::Standard(s) => Some(s.get_font_data()),
                FontQuery::Fallback(f) => Some(f.pick_standard_font().get_font_data()),
            }),
            ..Default::default()
        };
        Self {
            pdf,
            interpreter_settings,
        }
    }

    #[allow(dead_code)]
    #[cfg(not(feature = "pdfium"))] // Avoid compile error.
    pub fn render_pdf(&mut self, app_state: &mut AppState) {
        let render_settings = RenderSettings {
            x_scale: app_state.scale,
            y_scale: app_state.scale,
            bg_color: WHITE,
            ..Default::default()
        };

        let mut output: Vec<Pixmap> = Vec::new();

        if app_state.pages.is_none() {
            let cache = &RenderCache::new();
            let first_page = render(
                &self.pdf.pages()[0],
                cache,
                &self.interpreter_settings,
                &render_settings,
            );
            app_state.scales = Some(Vec::new());
            for _ in self.pdf.pages().iter() {
                let _ = &output.push(Pixmap::new(0, 0));
                app_state.scales.as_mut().unwrap().push(0.0);
            }
            output[0] = first_page;
            app_state.pages = Some(output);
            app_state.scales.as_mut().unwrap()[0] = 1.0;
        } else {
            let cache = &RenderCache::new();
            let rendered_page = render(
                &self.pdf.pages()[app_state.page],
                cache,
                &self.interpreter_settings,
                &render_settings,
            );
            app_state.pages.as_mut().unwrap()[app_state.page] = rendered_page;
        }
    }
}
