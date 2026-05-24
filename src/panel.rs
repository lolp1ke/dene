// SPDX-License-Identifier: Apache-2.0

use ratatui::layout::Rect;
use taffy::NodeId;

use crate::{AnyView, LayoutEngine};

#[derive(Debug)]
#[derive(Clone, Copy)]
#[derive(PartialEq)]
pub struct PanelId(pub(crate) u32);

#[derive(Debug)]
#[derive(Clone)]
pub struct Panel {
  pub id: PanelId,
  pub view: AnyView,
}

#[derive(Debug)]
#[derive(Clone)]
pub enum PanelNode {
  Leaf(Panel),
  Split {
    direction: Direction,
    children: Vec<PanelNode>,
    weights: Vec<f32>,
  },
}
impl PanelNode {
  pub(crate) fn visit_leaves<F>(
    &self,
    node_id: NodeId,
    engine: &LayoutEngine,
    offset_x: f32,
    offset_y: f32,
    f: &mut F,
  ) where
    F: FnMut(&Panel, Rect),
  {
    let layout = engine.layout(node_id);
    let abs_x = offset_x + layout.location.x;
    let abs_y = offset_y + layout.location.y;
    let width = layout.size.width.ceil() as u16;
    let height = layout.size.height.ceil() as u16;

    match self {
      Self::Leaf(pane) => {
        f(
          pane,
          Rect {
            x: abs_x as u16,
            y: abs_y as u16,
            width,
            height,
          },
        );
      }
      Self::Split { children, .. } => {
        let child_ids = engine.children(node_id);
        for (child, &child_id) in children.iter().zip(child_ids.iter()) {
          child.visit_leaves(child_id, engine, abs_x, abs_y, f);
        }
      }
    };
  }

  pub(crate) fn find(&self, id: PanelId) -> Option<&Panel> {
    match self {
      PanelNode::Leaf(pane) if pane.id == id => Some(pane),
      PanelNode::Leaf(..) => None,
      PanelNode::Split { children, .. } => {
        children.iter().find_map(|child| child.find(id))
      }
    }
  }

  pub fn tab_order(&self) -> Vec<PanelId> {
    #[inline]
    fn inner(this: &PanelNode, order: &mut Vec<PanelId>) {
      match this {
        PanelNode::Leaf(panel) => order.push(panel.id),
        PanelNode::Split { children, .. } => {
          for child in children.iter() {
            inner(child, order);
          }
        }
      };
    }

    let mut order = Vec::new();
    inner(self, &mut order);
    order
  }
}

#[derive(Debug)]
#[derive(Clone)]
pub enum Direction {
  Horizontal,
  Vertical,
}
