// SPDX-License-Identifier: Apache-2.0

use smallvec::SmallVec;

use crate::{
  AnyElement, App, Element, InteractiveElement, Interactivity, IntoElement,
  Rect, StyleableElement, Window,
};

#[derive(Debug)]
#[derive(Default)]
pub struct Div {
  interactivity: Interactivity,
  children: Vec<AnyElement>,
}
impl InteractiveElement for Div {
  fn interactivity(&mut self) -> &mut Interactivity {
    &mut self.interactivity
  }
}
impl Element for Div {
  type RequestLayoutState = SmallVec<[taffy::NodeId; 8]>;
  type PreRenderState = ();

  fn request_layout(
    &mut self,
    window: &mut Window,
    cx: &mut App,
  ) -> (taffy::NodeId, Self::RequestLayoutState) {
    let child_node_ids = self
      .children
      .iter_mut()
      .map(|child| child.request_layout(window, cx))
      .collect::<SmallVec<_>>();

    let node_id = window.request_layout(
      self.interactivity.base_style.clone(),
      &child_node_ids,
      cx,
    );
    (node_id, child_node_ids)
  }
  fn pre_render(
    &mut self,
    bounds: Rect,
    request_layout: &mut Self::RequestLayoutState,
    window: &mut Window,
    cx: &mut App,
  ) -> Self::PreRenderState {
    if matches!(self.interactivity.base_style.display, taffy::Display::None) {
      return;
    };

    for child in self.children.iter_mut() {
      child.pre_render(window, cx);
    }
  }
  fn render(
    &mut self,
    bounds: Rect,
    request_layout: &mut Self::RequestLayoutState,
    pre_render: &mut Self::PreRenderState,
    window: &mut Window,
    cx: &mut App,
  ) {
    if matches!(self.interactivity.base_style.display, taffy::Display::None) {
      return;
    };

    self.interactivity.apply_keyboard_listeners(window);
    for child in self.children.iter_mut() {
      child.render(window, cx);
    }
  }
}
impl IntoElement for Div {
  type Element = Self;

  fn into_element(self) -> Self::Element {
    self
  }
}
impl StyleableElement for Div {
  fn style(&mut self) -> &mut taffy::Style {
    &mut self.interactivity.base_style
  }
}

pub fn div() -> Div {
  Default::default()
}
