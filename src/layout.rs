// SPDX-License-Identifier: Apache-2.0

use taffy::{AvailableSpace, NodeId, Size, Style, TaffyTree};

use crate::Rect;

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

  pub(crate) fn clear(&mut self) {
    self.engine.clear();
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
  pub(crate) fn layout_bounds(&mut self, node_id: NodeId) -> Rect {
    let layout = self.engine.layout(node_id).unwrap();
    Rect {
      x: layout.location.x.ceil() as u16,
      y: layout.location.y.ceil() as u16,
      width: layout.size.width.ceil() as u16,
      height: layout.size.height.ceil() as u16,
    }
  }
  pub(crate) fn compute(
    &mut self,
    node_id: NodeId,
    available_space: Size<AvailableSpace>,
  ) {
    self
      .engine
      .compute_layout(node_id, available_space)
      .unwrap();
  }
}
