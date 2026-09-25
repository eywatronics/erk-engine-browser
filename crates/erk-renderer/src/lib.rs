//! Erk Engine renderer: layout, display list and paint.
//!
//! The public surface is deliberately small: HTML goes in, pixels come out.
//! DOM, style and layout types stay inside the crate, so the shell cannot
//! come to depend on them before the renderer moves into its own process.

mod color;
mod display;
mod layout;
mod paint;
mod text;

use std::sync::Arc;

use erk_dom::Document;
use erk_style::StyleEngine;
use vello_cpu::Pixmap;

use crate::display::DisplayList;
use crate::text::{EmbeddedFontMetrics, TextEngine};

/// A rendered page.
pub struct Frame {
    pixmap: Pixmap,
    display_list: String,
}

impl Frame {
    pub fn width(&self) -> u16 {
        self.pixmap.width()
    }

    pub fn height(&self) -> u16 {
        self.pixmap.height()
    }

    /// Pixels as premultiplied RGBA8, row by row.
    pub fn rgba(&self) -> &[u8] {
        self.pixmap.data_as_u8_slice()
    }

    /// The frame encoded as PNG.
    pub fn to_png(&self) -> Vec<u8> {
        self.pixmap
            .clone()
            .into_png()
            .expect("encoding an in-memory pixmap cannot fail")
    }

    /// The display list the frame was painted from, as text.
    pub fn display_list(&self) -> &str {
        &self.display_list
    }
}

/// Parse, style, lay out and paint `html` in a `width` × `height` viewport
/// of CSS pixels (1 CSS pixel = 1 device pixel in M0).
pub fn render_html(html: &str, width: u16, height: u16) -> Frame {
    let (w, h) = (f32::from(width), f32::from(height));
    let doc = Document::parse_html(html);
    let styles = StyleEngine::with_font_metrics(w, h, Arc::new(EmbeddedFontMetrics)).style(&doc);
    let mut text = TextEngine::new();
    let layouts = layout::layout(&doc, &styles, &mut text, w, h);
    let list = DisplayList::build(&doc, &styles, &layouts);
    Frame {
        pixmap: paint::paint(&list, width, height),
        display_list: list.dump(),
    }
}
