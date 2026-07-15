// SPDX-License-Identifier: Apache-2.0

use taffy::{NodeId, Style, TaffyTree};

#[derive(Debug)]
pub(crate) struct LayoutEngine {
  engine: TaffyTree<()>,
}
impl LayoutEngine {
  pub(crate) fn new() -> Self {
    Self {
      engine: TaffyTree::new(),
    }
  }

  pub(crate) fn request_layout(
    &mut self,
    style: Style,
    children: &[NodeId],
  ) -> NodeId {
    if children.is_empty() {
      self.engine.new_leaf(style).unwrap()
    } else {
      self.engine.new_with_children(style, children).unwrap()
    }
  }
}
