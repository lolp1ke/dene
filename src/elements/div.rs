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
    todo!();
  }
  fn pre_render(
    &mut self,
    bounds: Rect,
    window: &mut Window,
    cx: &mut App,
  ) -> Self::PreRenderState {
    todo!();
  }
  fn render(
    &mut self,
    bounds: Rect,
    request_layout: &mut Self::RequestLayoutState,
    pre_render: &mut Self::PreRenderState,
    window: &mut Window,
    cx: &mut App,
  ) {
    todo!();
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
