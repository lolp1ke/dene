// SPDX-License-Identifier: Apache-2.0

use taffy::{AvailableSpace, NodeId, Size, Style, TaffyTree};

use crate::Rect;

#[derive(derive_more::Debug)]
pub(crate) struct LayoutEngine {
  #[debug(skip)]
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
    let mut x = 0.;
    let mut y = 0.;
    let mut width = None;
    let mut height = None;
    let mut current_id = Some(node_id);

    while let Some(id) = current_id {
      let layout = self.engine.layout(id).unwrap();
      x += layout.location.x;
      y += layout.location.y;
      if width.is_none() && height.is_none() {
        width = Some(layout.size.width);
        height = Some(layout.size.height);
      };

      current_id = self.engine.parent(id);
    }

    let width = width.unwrap_or(0.).ceil() as u16;
    let height = height.unwrap_or(0.).ceil() as u16;
    Rect {
      x: x.ceil() as u16,
      y: y.ceil() as u16,
      width,
      height,
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

  pub(crate) fn content_box(&self, node_id: NodeId) -> Rect {
    let layout = self.engine.layout(node_id).unwrap();
    let left = (layout.border.left + layout.padding.left).ceil();
    let top = (layout.border.top + layout.padding.top).ceil();
    let right = (layout.border.right + layout.padding.right).ceil();
    let bottom = (layout.border.bottom + layout.padding.bottom).ceil();
    Rect {
      x: left as u16,
      y: top as u16,
      width: (layout.size.width.ceil() - left - right).max(0.) as u16,
      height: (layout.size.height.ceil() - top - bottom).max(0.) as u16,
    }
  }

  pub(crate) fn style(&self, node_id: NodeId) -> Style {
    self.engine.style(node_id).cloned().unwrap()
  }
  pub(crate) fn set_style(&mut self, node_id: NodeId, style: Style) {
    self.engine.set_style(node_id, style).unwrap();
  }
}
