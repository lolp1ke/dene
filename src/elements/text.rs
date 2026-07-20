// SPDX-License-Identifier: Apache-2.0

use crate::{App, Element, IntoElement, Rect, Window, get_terminal};

#[derive(Debug)]
pub struct Text {
  text: String,
}
impl Element for Text {
  type RequestLayoutState = ();
  type PreRenderState = ();

  fn request_layout(
    &mut self,
    window: &mut Window,
    cx: &mut App,
  ) -> (taffy::NodeId, Self::RequestLayoutState) {
    let width = self.text.len() as f32;
    let height = self.text.lines().count() as f32;
    let mut style = taffy::Style::DEFAULT;
    style.min_size = taffy::Size {
      width: taffy::Dimension::length(width),
      height: taffy::Dimension::length(height.max(1.)),
    };
    style.size = taffy::Size {
      width: taffy::Dimension::length(width),
      height: taffy::Dimension::length(height),
    };
    let node_id = window.request_layout(style, &[], cx);
    (node_id, ())
  }
  fn pre_render(
    &mut self,
    _: Rect,
    _: &mut Self::RequestLayoutState,
    _: &mut Window,
    _: &mut App,
  ) -> Self::PreRenderState {
  }
  fn render(
    &mut self,
    bounds: Rect,
    _: &mut Self::RequestLayoutState,
    _: &mut Self::PreRenderState,
    _: &mut Window,
    _: &mut App,
  ) {
    let lines = self.text.lines();
    let mut terminal = get_terminal().write();

    for (i, line) in lines.into_iter().enumerate() {
      let y = bounds.y + i as u16;
      if y >= bounds.y + bounds.height {
        break;
      };
      terminal.write_at(bounds.x, y, line);
    }
    // terminal.flush();
  }
}
impl IntoElement for Text {
  type Element = Self;
  fn into_element(self) -> Self::Element {
    self
  }
}
impl IntoElement for String {
  type Element = Text;
  fn into_element(self) -> Self::Element {
    Self::Element { text: self }
  }
}
impl IntoElement for &'_ str {
  type Element = Text;
  fn into_element(self) -> Self::Element {
    Self::Element {
      text: self.to_string(),
    }
  }
}
