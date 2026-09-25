//! Box layout with Taffy's low-level API.
//!
//! The DOM is the layout tree: Taffy walks Erk's nodes through the traits
//! below instead of a copy in a `TaffyTree`. Per-node layout state lives in a
//! side table indexed by `NodeId::index()`, like the style state in
//! erk-style. The trait implementations follow blitz-dom 0.3.0-beta.2,
//! src/layout/mod.rs (MIT OR Apache-2.0).
//!
//! M0 lays out element boxes only. Text arrives in Task 5 as paragraph leaves
//! measured by Parley; until then text contributes no size.

mod calc;

#[cfg(test)]
mod tests;

use erk_dom::{Document, NodeData, NodeId};
use erk_style::Styles;
use erk_style::style::Atom;
use taffy::{
    AvailableSpace, BlockContext, Cache, CacheTree, Display, Layout, LayoutBlockContainer,
    LayoutFlexboxContainer, LayoutGridContainer, LayoutInput, LayoutOutput, LayoutPartialTree,
    RoundTree, Size, Style, TraversePartialTree, TraverseTree, compute_block_layout,
    compute_cached_layout, compute_flexbox_layout, compute_grid_layout, compute_root_layout,
    round_layout,
};

use self::calc::CalcTable;

/// The result of laying out one document.
pub(crate) struct Layouts {
    nodes: Vec<Option<Layout>>,
}

impl Layouts {
    /// The final (pixel-rounded) layout of a box, relative to its parent box.
    /// `None` for nodes that generate no box.
    pub(crate) fn get(&self, id: NodeId) -> Option<&Layout> {
        self.nodes.get(id.index() as usize)?.as_ref()
    }
}

/// Lay out `doc` in a viewport of `width` × `height` CSS pixels.
pub(crate) fn layout(doc: &Document, styles: &Styles, width: f32, height: f32) -> Layouts {
    let mut tree = LayoutTree::build(doc, styles);
    let root = taffy_id(doc.root());
    compute_root_layout(
        &mut tree,
        root,
        Size {
            width: AvailableSpace::Definite(width),
            height: AvailableSpace::Definite(height),
        },
    );
    round_layout(&mut tree, root);
    Layouts {
        nodes: tree
            .nodes
            .into_iter()
            .map(|node| node.in_tree.then_some(node.layout))
            .collect(),
    }
}

#[derive(Default)]
struct LayoutNode {
    /// Whether this arena slot generates a box in the layout tree.
    in_tree: bool,
    children: Vec<taffy::NodeId>,
    style: Style<Atom>,
    cache: Cache,
    unrounded: Layout,
    layout: Layout,
}

struct LayoutTree {
    nodes: Vec<LayoutNode>,
    calcs: CalcTable,
}

impl LayoutTree {
    fn build(doc: &Document, styles: &Styles) -> Self {
        let mut nodes: Vec<LayoutNode> = (0..doc.capacity_hint())
            .map(|_| LayoutNode::default())
            .collect();
        let mut calcs = CalcTable::default();

        // The document node is the initial containing block's box.
        let root = &mut nodes[doc.root().index() as usize];
        root.in_tree = true;
        root.style = Style {
            display: Display::Block,
            ..Style::DEFAULT
        };

        let mut stack = vec![doc.root()];
        while let Some(parent) = stack.pop() {
            let mut children = Vec::new();
            for child in doc.children(parent) {
                let is_element = matches!(
                    doc.node(child).map(|node| &node.data),
                    Some(NodeData::Element(_))
                );
                // Elements inside display:none are not styled, so a missing
                // style means no box.
                let Some(computed) = is_element.then(|| styles.computed(child)).flatten() else {
                    continue;
                };
                let style = stylo_taffy::to_taffy_style(&computed);
                if style.display == Display::None {
                    continue;
                }
                calcs.record(&computed);
                let node = &mut nodes[child.index() as usize];
                node.in_tree = true;
                node.style = style;
                children.push(taffy_id(child));
                stack.push(child);
            }
            nodes[parent.index() as usize].children = children;
        }

        Self { nodes, calcs }
    }

    fn node(&self, id: taffy::NodeId) -> &LayoutNode {
        &self.nodes[usize::from(id)]
    }

    fn node_mut(&mut self, id: taffy::NodeId) -> &mut LayoutNode {
        &mut self.nodes[usize::from(id)]
    }

    fn compute_child_layout_internal(
        &mut self,
        id: taffy::NodeId,
        inputs: LayoutInput,
        block_ctx: Option<&mut BlockContext<'_>>,
    ) -> LayoutOutput {
        match self.node(id).style.display {
            Display::Block => compute_block_layout(self, id, inputs, block_ctx),
            // A flow root establishes a new block formatting context: floats
            // and margins do not cross it.
            Display::FlowRoot => compute_block_layout(self, id, inputs, None),
            Display::Flex => compute_flexbox_layout(self, id, inputs),
            Display::Grid => compute_grid_layout(self, id, inputs),
            Display::None => LayoutOutput::HIDDEN,
        }
    }
}

fn taffy_id(id: NodeId) -> taffy::NodeId {
    taffy::NodeId::from(id.index() as usize)
}

impl TraversePartialTree for LayoutTree {
    type ChildIter<'a> = std::iter::Copied<std::slice::Iter<'a, taffy::NodeId>>;

    fn child_ids(&self, parent: taffy::NodeId) -> Self::ChildIter<'_> {
        self.node(parent).children.iter().copied()
    }

    fn child_count(&self, parent: taffy::NodeId) -> usize {
        self.node(parent).children.len()
    }

    fn get_child_id(&self, parent: taffy::NodeId, index: usize) -> taffy::NodeId {
        self.node(parent).children[index]
    }
}

impl TraverseTree for LayoutTree {}

impl LayoutPartialTree for LayoutTree {
    type CoreContainerStyle<'a> = &'a Style<Atom>;
    type CustomIdent = Atom;

    fn get_core_container_style(&self, id: taffy::NodeId) -> &Style<Atom> {
        &self.node(id).style
    }

    fn resolve_calc_value(&self, ptr: *const (), basis: f32) -> f32 {
        self.calcs.resolve(ptr, basis)
    }

    fn set_unrounded_layout(&mut self, id: taffy::NodeId, layout: &Layout) {
        self.node_mut(id).unrounded = *layout;
    }

    fn compute_child_layout(&mut self, id: taffy::NodeId, inputs: LayoutInput) -> LayoutOutput {
        compute_cached_layout(self, id, inputs, |tree, id, inputs| {
            tree.compute_child_layout_internal(id, inputs, None)
        })
    }
}

impl CacheTree for LayoutTree {
    fn cache_get(&mut self, id: taffy::NodeId, inputs: &LayoutInput) -> Option<LayoutOutput> {
        self.node_mut(id).cache.get(inputs)
    }

    fn cache_store(&mut self, id: taffy::NodeId, inputs: &LayoutInput, output: LayoutOutput) {
        self.node_mut(id).cache.store(inputs, output);
    }

    fn cache_clear(&mut self, id: taffy::NodeId) {
        self.node_mut(id).cache.clear();
    }
}

impl LayoutBlockContainer for LayoutTree {
    type BlockContainerStyle<'a> = &'a Style<Atom>;
    type BlockItemStyle<'a> = &'a Style<Atom>;

    fn get_block_container_style(&self, id: taffy::NodeId) -> &Style<Atom> {
        &self.node(id).style
    }

    fn get_block_child_style(&self, id: taffy::NodeId) -> &Style<Atom> {
        &self.node(id).style
    }

    fn compute_block_child_layout(
        &mut self,
        id: taffy::NodeId,
        inputs: LayoutInput,
        block_ctx: Option<&mut BlockContext<'_>>,
    ) -> LayoutOutput {
        compute_cached_layout(self, id, inputs, |tree, id, inputs| {
            tree.compute_child_layout_internal(id, inputs, block_ctx)
        })
    }
}

impl LayoutFlexboxContainer for LayoutTree {
    type FlexboxContainerStyle<'a> = &'a Style<Atom>;
    type FlexboxItemStyle<'a> = &'a Style<Atom>;

    fn get_flexbox_container_style(&self, id: taffy::NodeId) -> &Style<Atom> {
        &self.node(id).style
    }

    fn get_flexbox_child_style(&self, id: taffy::NodeId) -> &Style<Atom> {
        &self.node(id).style
    }
}

impl LayoutGridContainer for LayoutTree {
    type GridContainerStyle<'a> = &'a Style<Atom>;
    type GridItemStyle<'a> = &'a Style<Atom>;

    fn get_grid_container_style(&self, id: taffy::NodeId) -> &Style<Atom> {
        &self.node(id).style
    }

    fn get_grid_child_style(&self, id: taffy::NodeId) -> &Style<Atom> {
        &self.node(id).style
    }
}

impl RoundTree for LayoutTree {
    fn get_unrounded_layout(&self, id: taffy::NodeId) -> Layout {
        self.node(id).unrounded
    }

    fn set_final_layout(&mut self, id: taffy::NodeId, layout: &Layout) {
        self.node_mut(id).layout = *layout;
    }
}
