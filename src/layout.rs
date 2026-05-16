use ratatui::layout::Rect;
use taffy::{
  AvailableSpace, Display, FlexDirection, Layout, NodeId, Size, Style,
  TaffyTree, prelude::percent,
};

use crate::{Direction, PanelNode};

#[derive(Debug)]
pub struct LayoutEngine {
  taffy: TaffyTree<()>,
}
impl LayoutEngine {
  pub fn new() -> Self {
    Self {
      taffy: TaffyTree::new(),
    }
  }

  pub fn layout(&self, node_id: NodeId) -> &Layout {
    self.taffy.layout(node_id).unwrap()
  }
  pub fn children(&self, node_id: NodeId) -> Vec<NodeId> {
    self.taffy.children(node_id).unwrap()
  }

  pub fn compute(&mut self, root_id: NodeId, width: f32, height: f32) {
    self
      .taffy
      .compute_layout(
        root_id,
        Size {
          width: AvailableSpace::Definite(width),
          height: AvailableSpace::Definite(height),
        },
      )
      .unwrap();
  }
  pub fn build(&mut self, node: &PanelNode) -> NodeId {
    self.taffy.clear();
    let root_id = self.build_node(node);
    let mut style = self.taffy.style(root_id).unwrap().clone();
    style.size = Size {
      width: percent(1.0),
      height: percent(1.0),
    };
    self.taffy.set_style(root_id, style).unwrap();
    root_id
  }
  fn build_node(&mut self, node: &PanelNode) -> NodeId {
    match node {
      PanelNode::Leaf(..) => self
        .taffy
        .new_leaf(Style {
          display: Display::Flex,
          flex_grow: 1.0,
          flex_shrink: 1.0,
          ..Default::default()
        })
        .unwrap(),
      PanelNode::Split {
        direction,
        children,
        weights,
      } => {
        let total_weight = weights.iter().sum::<f32>();
        let child_ids = children
          .iter()
          .zip(weights.iter())
          .map(|(child, &weight)| {
            let id = self.build_node(child);
            let mut style = self.taffy.style(id).unwrap().clone();
            style.flex_grow = weight / total_weight;
            style.flex_shrink = 1.0;
            self.taffy.set_style(id, style).unwrap();
            id
          })
          .collect::<Vec<_>>();

        self
          .taffy
          .new_with_children(
            Style {
              display: Display::Flex,
              flex_direction: match direction {
                Direction::Horizontal => FlexDirection::Row,
                Direction::Vertical => FlexDirection::Column,
              },
              flex_grow: 1.0,
              flex_shrink: 1.0,
              ..Default::default()
            },
            &child_ids,
          )
          .unwrap()
      }
    }
  }
}
impl Default for LayoutEngine {
  fn default() -> Self {
    Self::new()
  }
}
