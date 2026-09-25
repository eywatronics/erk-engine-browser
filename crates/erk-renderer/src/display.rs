//! The display list: what to paint, in paint order, in absolute coordinates.
//!
//! Items are flat and own their data, so a list can later cross a process
//! boundary to a compositor. M0 has backgrounds and glyph runs; borders,
//! images, clips and the spatial tree come with the features that need them.

use std::fmt::Write as _;

use erk_dom::{Document, NodeId, local_name};
use erk_style::{ComputedValues, Styles};
use parley::{FontData, PositionedLayoutItem};

use crate::color::{Rgba, srgb_bytes};
use crate::layout::Layouts;

pub(crate) struct DisplayList {
    /// The canvas colour behind everything (CSS 2 §14.2).
    pub(crate) canvas: Rgba,
    pub(crate) items: Vec<DisplayItem>,
}

pub(crate) enum DisplayItem {
    Rect {
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        color: Rgba,
    },
    Glyphs(GlyphRun),
}

pub(crate) struct GlyphRun {
    pub(crate) font: FontData,
    pub(crate) size: f32,
    pub(crate) color: Rgba,
    pub(crate) glyphs: Vec<PositionedGlyph>,
    /// The source text, for dumps and debugging.
    pub(crate) text: String,
}

#[derive(Clone, Copy)]
pub(crate) struct PositionedGlyph {
    pub(crate) id: u32,
    pub(crate) x: f32,
    pub(crate) y: f32,
}

const WHITE: Rgba = [255, 255, 255, 255];

impl DisplayList {
    pub(crate) fn build(doc: &Document, styles: &Styles, layouts: &Layouts) -> Self {
        let mut list = Self {
            canvas: WHITE,
            items: Vec::new(),
        };
        let canvas_source = list.propagate_canvas_background(doc, styles);
        list.add_box(doc, styles, layouts, doc.root(), (0.0, 0.0), canvas_source);
        list
    }

    /// The root element's background paints the whole canvas; if it has
    /// none, the body's does (CSS 2 §14.2). Returns the element whose
    /// background was used, so it is not painted a second time.
    fn propagate_canvas_background(&mut self, doc: &Document, styles: &Styles) -> Option<NodeId> {
        let html = child_element(doc, doc.root(), &local_name!("html"))?;
        let body = child_element(doc, html, &local_name!("body"));
        for id in std::iter::once(html).chain(body) {
            let style = styles.computed(id)?;
            let color = background(&style);
            if color[3] != 0 {
                self.canvas = color;
                return Some(id);
            }
        }
        None
    }

    fn add_box(
        &mut self,
        doc: &Document,
        styles: &Styles,
        layouts: &Layouts,
        id: NodeId,
        parent_origin: (f32, f32),
        canvas_source: Option<NodeId>,
    ) {
        let Some(layout) = layouts.get(id) else {
            return;
        };
        let x = parent_origin.0 + layout.location.x;
        let y = parent_origin.1 + layout.location.y;

        if let Some(style) = styles.computed(id)
            && Some(id) != canvas_source
        {
            let color = background(&style);
            if color[3] != 0 {
                self.items.push(DisplayItem::Rect {
                    x,
                    y,
                    width: layout.size.width,
                    height: layout.size.height,
                    color,
                });
            }
        }

        if let Some(shaped) = layouts.text(id) {
            let content_x = x + layout.border.left + layout.padding.left;
            let content_y = y + layout.border.top + layout.padding.top;
            self.add_text(&shaped.text, &shaped.layout, (content_x, content_y));
        }

        for child in doc.children(id) {
            self.add_box(doc, styles, layouts, child, (x, y), canvas_source);
        }
    }

    fn add_text(
        &mut self,
        text: &str,
        layout: &parley::Layout<crate::text::TextBrush>,
        origin: (f32, f32),
    ) {
        for line in layout.lines() {
            for item in line.items() {
                let PositionedLayoutItem::GlyphRun(run) = item else {
                    continue;
                };
                let glyphs = run
                    .positioned_glyphs()
                    .map(|glyph| PositionedGlyph {
                        id: glyph.id,
                        x: origin.0 + glyph.x,
                        y: origin.1 + glyph.y,
                    })
                    .collect();
                let range = run.run().text_range();
                self.items.push(DisplayItem::Glyphs(GlyphRun {
                    font: run.run().font().clone(),
                    size: run.run().font_size(),
                    color: run.style().brush.0,
                    glyphs,
                    text: text.get(range).unwrap_or_default().to_owned(),
                }));
            }
        }
    }

    /// A readable, stable text form of the list, one item per line. Useful
    /// when a golden image changes and the question is what moved.
    pub(crate) fn dump(&self) -> String {
        let mut out = format!("canvas {}\n", hex(self.canvas));
        for item in &self.items {
            match item {
                DisplayItem::Rect {
                    x,
                    y,
                    width,
                    height,
                    color,
                } => {
                    let _ = writeln!(out, "rect {x} {y} {width}x{height} {}", hex(*color));
                }
                DisplayItem::Glyphs(run) => {
                    let (x, y) = run.glyphs.first().map_or((0.0, 0.0), |g| (g.x, g.y));
                    let _ = writeln!(
                        out,
                        "glyphs {x} {y} {}px {} {:?}",
                        run.size,
                        hex(run.color),
                        run.text
                    );
                }
            }
        }
        out
    }
}

fn background(style: &ComputedValues) -> Rgba {
    srgb_bytes(style.resolve_color(&style.get_background().background_color))
}

fn child_element(doc: &Document, parent: NodeId, name: &erk_dom::LocalName) -> Option<NodeId> {
    doc.children(parent).find(|&child| {
        doc.node(child)
            .and_then(|node| node.as_element())
            .is_some_and(|element| element.name.local == *name)
    })
}

fn hex([r, g, b, a]: Rgba) -> String {
    format!("#{r:02x}{g:02x}{b:02x}{a:02x}")
}
