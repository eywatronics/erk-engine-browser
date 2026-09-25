use erk_dom::{Document, LocalName, NodeId, local_name};
use erk_style::StyleEngine;
use taffy::Layout;

use super::{Layouts, layout};

const WIDTH: f32 = 800.0;
const HEIGHT: f32 = 600.0;

/// Lay out `body` with the UA body margin removed, so positions inside it are
/// easy to read.
fn lay_out(body: &str) -> (Document, Layouts) {
    let html = format!("<style>body {{ margin: 0 }}</style><body>{body}</body>");
    let doc = Document::parse_html(&html);
    let styles = StyleEngine::new(WIDTH, HEIGHT).style(&doc);
    let layouts = layout(&doc, &styles, WIDTH, HEIGHT);
    (doc, layouts)
}

/// Elements with the given tag, in tree order.
fn all(doc: &Document, tag: &LocalName) -> Vec<NodeId> {
    let mut found = Vec::new();
    let mut stack = vec![doc.root()];
    while let Some(id) = stack.pop() {
        if doc
            .node(id)
            .and_then(|node| node.as_element())
            .is_some_and(|element| element.name.local == *tag)
        {
            found.push(id);
        }
        let mut children: Vec<_> = doc.children(id).collect();
        children.reverse();
        stack.extend(children);
    }
    found
}

fn divs(doc: &Document, layouts: &Layouts) -> Vec<Layout> {
    all(doc, &local_name!("div"))
        .into_iter()
        .map(|id| *layouts.get(id).expect("div has a box"))
        .collect()
}

#[test]
fn explicit_width_is_used() {
    let (doc, layouts) = lay_out(r#"<div style="width: 100px; height: 10px"></div>"#);
    let div = divs(&doc, &layouts)[0];
    assert_eq!((div.size.width, div.size.height), (100.0, 10.0));
}

#[test]
fn a_block_without_width_fills_its_container() {
    let (doc, layouts) = lay_out(r#"<div style="height: 10px"></div>"#);
    assert_eq!(divs(&doc, &layouts)[0].size.width, WIDTH);
}

#[test]
fn block_siblings_stack_vertically() {
    let (doc, layouts) =
        lay_out(r#"<div style="height: 50px"></div><div style="height: 30px"></div>"#);
    let [first, second] = divs(&doc, &layouts)[..] else {
        panic!("expected two divs");
    };
    assert_eq!(first.location.y, 0.0);
    assert_eq!(second.location.y, 50.0);

    let body = all(&doc, &local_name!("body"))[0];
    assert_eq!(layouts.get(body).unwrap().size.height, 80.0);
}

#[test]
fn margin_top_offsets_the_box() {
    let (doc, layouts) = lay_out(
        r#"<div style="height: 10px"></div><div style="margin-top: 20px; height: 10px"></div>"#,
    );
    assert_eq!(divs(&doc, &layouts)[1].location.y, 30.0);
}

#[test]
fn adjacent_vertical_margins_collapse() {
    // CSS 2 §8.3.1: the gap between siblings is max(20, 30) = 30, not 50.
    let (doc, layouts) = lay_out(
        r#"<div style="margin-bottom: 20px; height: 10px"></div>
           <div style="margin-top: 30px; height: 10px"></div>"#,
    );
    assert_eq!(divs(&doc, &layouts)[1].location.y, 40.0);
}

#[test]
fn display_none_generates_no_box() {
    let (doc, layouts) = lay_out(
        r#"<div style="display: none; height: 50px"></div><div style="height: 10px"></div>"#,
    );
    let ids = all(&doc, &local_name!("div"));
    assert!(layouts.get(ids[0]).is_none());
    assert_eq!(layouts.get(ids[1]).unwrap().location.y, 0.0);
}

#[test]
fn calc_widths_resolve_against_the_container() {
    let (doc, layouts) = lay_out(r#"<div style="width: calc(50% - 20px); height: 10px"></div>"#);
    assert_eq!(divs(&doc, &layouts)[0].size.width, WIDTH / 2.0 - 20.0);
}

#[test]
fn padding_and_border_widen_the_border_box() {
    let (doc, layouts) = lay_out(
        r#"<div style="width: 100px; height: 10px; padding: 5px; border: 2px solid"></div>"#,
    );
    let div = divs(&doc, &layouts)[0];
    assert_eq!(div.size.width, 100.0 + 2.0 * 5.0 + 2.0 * 2.0);
    assert_eq!(div.padding.left, 5.0);
    assert_eq!(div.border.left, 2.0);
}
