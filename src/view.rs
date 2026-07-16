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
  type RequestLayoutState = Option<AnyElement>;
  type PreRenderState = Option<AnyElement>;

  fn request_layout(
    &mut self,
    window: &mut Window,
    cx: &mut App,
  ) -> (taffy::NodeId, Self::RequestLayoutState) {
    let mut element = (self.render)(self, window, cx);
    let node_id = element.request_layout(window, cx);
    (node_id, Some(element))
  }
  fn pre_render(
    &mut self,
    _: Rect,
    request_layout: &mut Self::RequestLayoutState,
    window: &mut Window,
    cx: &mut App,
  ) -> Self::PreRenderState {
    if let Some(mut element) = request_layout.take() {
      element.pre_render(window, cx);
      return Some(element);
    };
    None
  }
  fn render(
    &mut self,
    _: Rect,
    _: &mut Self::RequestLayoutState,
    pre_render: &mut Self::PreRenderState,
    window: &mut Window,
    cx: &mut App,
  ) {
    pre_render.as_mut().unwrap().render(window, cx);
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
