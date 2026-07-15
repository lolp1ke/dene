// SPDX-License-Identifier: Apache-2.0

use crate::{
  AnyElement, AnyEntity, App, Element, Entity, IntoElement, Rect, Render,
  Window,
};

#[derive(Debug)]
#[derive(Clone)]
pub struct AnyView {
  entity: AnyEntity,
  render: fn(&Self, &mut Window, &mut App) -> AnyElement,
}
impl AnyView {
  fn downcast<E>(self) -> Option<Entity<E>>
  where
    E: 'static,
  {
    self.entity.downcast()
  }
}
impl<V> From<Entity<V>> for AnyView
where
  V: Render,
{
  fn from(value: Entity<V>) -> Self {
    Self {
      entity: value.into_any(),
      render: render::<V>,
    }
  }
}
impl Element for AnyView {
  type RequestLayoutState = ();
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
impl IntoElement for AnyView {
  type Element = Self;
  fn into_element(self) -> Self::Element {
    self
  }
}

fn render<V>(
  any_view: &AnyView,
  window: &mut Window,
  cx: &mut App,
) -> AnyElement
where
  V: Render,
{
  let view = any_view.clone().downcast::<V>().unwrap();
  view.update(cx, |view, cx| view.render(window, cx).into_any_element())
}
