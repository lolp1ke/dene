// SPDX-License-Identifier: Apache-2.0

use std::{cell::RefCell, ops::Range, rc::Rc};

use smallvec::SmallVec;

use crate::{
  AnyElement, App, Axis, Context, Div, Element, Entity, Hitbox,
  InteractiveElement, IntoElement, Rect, Render, ScrollHandle, Size,
  StyleRefinement, StyleableElement, Window, div,
};

type RenderItemsFn = dyn for<'a> Fn(
  Range<usize>,
  &'a mut Window,
  &'a mut App,
) -> SmallVec<[AnyElement; 16]>;

pub struct VirtualListRequestLayoutState {
  items: SmallVec<[AnyElement; 16]>,
}

#[derive(derive_more::Debug)]
pub struct VirtualList {
  base: Div,
  axis: Axis,
  item_sizes: Rc<Vec<Size>>,
  scroll_handle: VirtualListScrollHandle,
  #[debug(skip)]
  render_items: Box<RenderItemsFn>,
}
impl VirtualList {}
impl StyleableElement for VirtualList {
  #[inline(always)]
  fn style(&mut self) -> &mut StyleRefinement {
    self.base.style()
  }
}
impl IntoElement for VirtualList {
  type Element = Self;

  fn into_element(self) -> Self::Element {
    self
  }
}
impl Element for VirtualList {
  type RequestLayoutState = VirtualListRequestLayoutState;
  type PreRenderState = Option<Hitbox>;

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
    request_layout: &mut Self::RequestLayoutState,
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

#[inline(always)]
fn v_virtual_list<V, F, R>(
  view: &Entity<V>,
  item_sizes: Rc<Vec<Size>>,
  f: F,
) -> VirtualList
where
  V: Render,
  F: 'static + Fn(&mut V, Range<usize>, &mut Window, &mut Context<V>) -> Vec<R>,
  R: IntoElement,
{
  virtual_list(view, Axis::Vertical, item_sizes, f)
}
#[inline(always)]
fn h_virtual_list<V, F, R>(
  view: &Entity<V>,
  item_sizes: Rc<Vec<Size>>,
  f: F,
) -> VirtualList
where
  V: Render,
  F: 'static + Fn(&mut V, Range<usize>, &mut Window, &mut Context<V>) -> Vec<R>,
  R: IntoElement,
{
  virtual_list(view, Axis::Horizontal, item_sizes, f)
}
fn virtual_list<V, F, R>(
  view: &Entity<V>,
  axis: Axis,
  item_sizes: Rc<Vec<Size>>,
  f: F,
) -> VirtualList
where
  V: Render,
  F: 'static + Fn(&mut V, Range<usize>, &mut Window, &mut Context<V>) -> Vec<R>,
  R: IntoElement,
{
  let view = view.clone();
  let scroll_handle = VirtualListScrollHandle::new();
  let render_range = move |range, window: &mut Window, cx: &mut App| {
    view
      .update(cx, |this, cx| f(this, range, window, cx))
      .into_iter()
      .map(IntoElement::into_any_element)
      .collect()
  };

  VirtualList {
    base: div().size_full().track_scroll(&scroll_handle),
    axis,
    item_sizes,
    scroll_handle,
    render_items: Box::new(render_range),
  }
}

#[derive(Debug)]
struct VirtualListScrollHandleInner {
  axis: Axis,
  items_count: usize,
}
#[derive(Debug)]
#[derive(derive_more::Deref)]
pub struct VirtualListScrollHandle {
  inner: Rc<RefCell<VirtualListScrollHandleInner>>,
  #[deref]
  scroll_handle: ScrollHandle,
}
impl VirtualListScrollHandle {
  pub fn new() -> Self {
    Self {
      inner: Rc::new(RefCell::new(VirtualListScrollHandleInner {
        axis: Axis::Vertical,
        items_count: 0,
      })),
      scroll_handle: ScrollHandle::default(),
    }
  }
}
impl AsRef<ScrollHandle> for VirtualListScrollHandle {
  fn as_ref(&self) -> &ScrollHandle {
    &self.scroll_handle
  }
}
impl Default for VirtualListScrollHandle {
  fn default() -> Self {
    Self::new()
  }
}
