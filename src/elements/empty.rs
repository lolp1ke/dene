// SPDX-License-Identifier: Apache-2.0

use crate::{App, Context, Element, IntoElement, Rect, Render, Window};

#[derive(Debug)]
pub struct Empty;
impl Render for Empty {
  fn render(
    &mut self,
    _: &mut Window,
    _: &mut Context<Self>,
  ) -> impl IntoElement {
    Self
  }
}
impl Element for Empty {
  type RequestLayoutState = ();
  type PreRenderState = ();

  fn request_layout(
    &mut self,
    window: &mut Window,
    cx: &mut App,
  ) -> (taffy::NodeId, Self::RequestLayoutState) {
    (
      window.request_layout(
        taffy::Style {
          display: taffy::Display::None,
          ..Default::default()
        },
        &[],
        cx,
      ),
      (),
    )
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
    _: Rect,
    _: &mut Self::RequestLayoutState,
    _: &mut Self::PreRenderState,
    _: &mut Window,
    _: &mut App,
  ) {
  }
}
impl IntoElement for Empty {
  type Element = Self;
  fn into_element(self) -> Self::Element {
    self
  }
}
