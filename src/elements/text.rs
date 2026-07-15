// SPDX-License-Identifier: Apache-2.0

use crate::{Element, IntoElement};

#[derive(Debug)]
pub struct Text {
  text: String,
}
impl Element for Text {
  type RequestLayoutState = ();
  type PreRenderState = ();

  fn request_layout(
    &mut self,
    window: &mut crate::Window,
    cx: &mut crate::App,
  ) -> (taffy::NodeId, Self::RequestLayoutState) {
    todo!();
  }
  fn pre_render(
    &mut self,
    bounds: crate::Rect,
    request_layout: &mut Self::RequestLayoutState,
    window: &mut crate::Window,
    cx: &mut crate::App,
  ) -> Self::PreRenderState {
    todo!();
  }
  fn render(
    &mut self,
    bounds: crate::Rect,
    request_layout: &mut Self::RequestLayoutState,
    pre_render: &mut Self::PreRenderState,
    window: &mut crate::Window,
    cx: &mut crate::App,
  ) {
    todo!();
  }
}
impl IntoElement for Text {
  type Element = Self;
  fn into_element(self) -> Self::Element {
    self
  }
}
