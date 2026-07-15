// SPDX-License-Identifier: Apache-2.0

use std::{fmt::Debug, mem};

use crate::{App, Context, Rect, Window};

pub trait Render: 'static + Sized {
  fn render(
    &mut self,
    window: &mut Window,
    cx: &mut Context<Self>,
  ) -> impl IntoElement;
}

pub trait IntoElement: Sized {
  type Element: Element;

  fn into_element(self) -> Self::Element;
  fn into_any_element(self) -> AnyElement {
    self.into_element().into_any()
  }
}
pub trait Element: 'static + IntoElement {
  type RequestLayoutState;
  type PreRenderState;

  fn request_layout(
    &mut self,
    window: &mut Window,
    cx: &mut App,
  ) -> (taffy::NodeId, Self::RequestLayoutState);
  fn pre_render(
    &mut self,
    bounds: Rect,
    window: &mut Window,
    cx: &mut App,
  ) -> Self::PreRenderState;
  fn render(
    &mut self,
    bounds: Rect,
    request_layout: &mut Self::RequestLayoutState,
    pre_render: &mut Self::PreRenderState,
    window: &mut Window,
    cx: &mut App,
  );

  fn into_any(self) -> AnyElement
  where
    Self: Sized,
  {
    AnyElement(Box::new(DrawableObject::new(self)))
  }
}

#[derive(Debug)]
pub struct AnyElement(Box<dyn ElementObject>);
impl AnyElement {
  fn request_layout(
    &mut self,
    window: &mut Window,
    cx: &mut App,
  ) -> taffy::NodeId {
    self.0.request_layout(window, cx)
  }
  fn pre_render(&mut self, window: &mut Window, cx: &mut App) {
    self.0.pre_render(window, cx);
  }
  fn render(&mut self, window: &mut Window, cx: &mut App) {
    self.0.render(window, cx);
  }
}
impl Element for AnyElement {
  type RequestLayoutState = ();
  type PreRenderState = ();

  fn request_layout(
    &mut self,
    window: &mut Window,
    cx: &mut App,
  ) -> (taffy::NodeId, Self::RequestLayoutState) {
    let node_id = self.request_layout(window, cx);
    (node_id, ())
  }
  fn pre_render(
    &mut self,
    _: Rect,
    window: &mut Window,
    cx: &mut App,
  ) -> Self::PreRenderState {
    self.pre_render(window, cx);
  }
  fn render(
    &mut self,
    _: Rect,
    _: &mut Self::RequestLayoutState,
    _: &mut Self::PreRenderState,
    window: &mut Window,
    cx: &mut App,
  ) {
    self.render(window, cx);
  }
}
impl IntoElement for AnyElement {
  type Element = Self;
  fn into_element(self) -> Self::Element {
    self
  }
  fn into_any_element(self) -> AnyElement {
    self
  }
}

struct DrawableObject<E>
where
  E: Element,
{
  element: E,
  phase: DrawableObjectPhase<E::RequestLayoutState, E::PreRenderState>,
}
impl<E> DrawableObject<E>
where
  E: Element,
{
  fn new(element: E) -> Self {
    Self {
      element,
      phase: DrawableObjectPhase::Start,
    }
  }

  fn request_layout(
    &mut self,
    window: &mut Window,
    cx: &mut App,
  ) -> taffy::NodeId {
    match mem::take(&mut self.phase) {
      DrawableObjectPhase::Start => {
        todo!()
      }
      _ => panic!("MUST BE CALLED ONCE"),
    }
  }
  fn pre_render(&mut self, window: &mut Window, cx: &mut App) {
    match mem::take(&mut self.phase) {
      DrawableObjectPhase::RequestLayout {
        node_id,
        request_layout,
      } => {}
      _ => panic!("MUST BE CALLED AFTER `request_layout`"),
    }
  }
  fn render(
    &mut self,
    window: &mut Window,
    cx: &mut App,
  ) -> (E::RequestLayoutState, E::RequestLayoutState) {
    match mem::take(&mut self.phase) {
      DrawableObjectPhase::PreRender {
        node_id,
        bounds,
        request_layout,
        pre_render,
      } => {
        self.phase = DrawableObjectPhase::Rendered;
        todo!();
      }
      _ => panic!("MUST BE CALLED AFTER `pre_render`"),
    }
  }
}
impl<E> ElementObject for DrawableObject<E>
where
  E: Element,
{
  fn request_layout(
    &mut self,
    window: &mut Window,
    cx: &mut App,
  ) -> taffy::NodeId {
    DrawableObject::request_layout(self, window, cx)
  }
  fn pre_render(&mut self, window: &mut Window, cx: &mut App) {
    DrawableObject::pre_render(self, window, cx);
  }
  fn render(&mut self, window: &mut Window, cx: &mut App) {
    DrawableObject::render(self, window, cx);
  }
}

#[derive(Default)]
enum DrawableObjectPhase<RequestLayoutState, PreRenderState> {
  #[default]
  Start,
  RequestLayout {
    node_id: taffy::NodeId,
    request_layout: RequestLayoutState,
  },
  PreRender {
    node_id: taffy::NodeId,
    bounds: Rect,
    request_layout: RequestLayoutState,
    pre_render: PreRenderState,
  },
  Rendered,
}

pub(crate) trait ElementObject {
  fn request_layout(
    &mut self,
    window: &mut Window,
    cx: &mut App,
  ) -> taffy::NodeId;
  fn pre_render(&mut self, window: &mut Window, cx: &mut App);
  fn render(&mut self, window: &mut Window, cx: &mut App);
}
impl Debug for dyn ElementObject {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_tuple("dyn ElementObject").finish_non_exhaustive()
  }
}

#[derive(derive_more::Debug)]
#[derive(Default)]
pub struct Interactivity {
  #[debug(skip)]
  pub(crate) base_style: taffy::Style,
}
pub trait InteractiveElement {
  fn interactivity(&mut self) -> &mut Interactivity;
}

pub trait StyleableElement: Sized {
  fn style(&mut self) -> &mut taffy::Style;

  fn block(mut self) -> Self {
    self.style().display = taffy::Display::Block;
    self
  }
  fn flex(mut self) -> Self {
    self.style().display = taffy::Display::Flex;
    self
  }
  fn grid(mut self) -> Self {
    self.style().display = taffy::Display::Grid;
    self
  }
  fn none(mut self) -> Self {
    self.style().display = taffy::Display::None;
    self
  }

  fn box_border(mut self) -> Self {
    self.style().box_sizing = taffy::BoxSizing::BorderBox;
    self
  }
  fn box_content(mut self) -> Self {
    self.style().box_sizing = taffy::BoxSizing::ContentBox;
    self
  }

  fn relative(mut self) -> Self {
    self.style().position = taffy::Position::Relative;
    self
  }
  fn absolute(mut self) -> Self {
    self.style().position = taffy::Position::Absolute;
    self
  }

  fn m(mut self, value: f32) -> Self {
    self.style().margin = taffy::Rect::length(value);
    self
  }
  fn m_auto(mut self) -> Self {
    self.style().margin = taffy::Rect::auto();
    self
  }
  fn mx(mut self, x: f32) -> Self {
    self.style().margin.left = taffy::LengthPercentageAuto::length(x);
    self.style().margin.right = taffy::LengthPercentageAuto::length(x);
    self
  }
  fn my(mut self, y: f32) -> Self {
    self.style().margin.bottom = taffy::LengthPercentageAuto::length(y);
    self.style().margin.top = taffy::LengthPercentageAuto::length(y);
    self
  }
  fn ml(mut self, l: f32) -> Self {
    self.style().margin.left = taffy::LengthPercentageAuto::length(l);
    self
  }
  fn mr(mut self, r: f32) -> Self {
    self.style().margin.right = taffy::LengthPercentageAuto::length(r);
    self
  }
  fn mt(mut self, t: f32) -> Self {
    self.style().margin.top = taffy::LengthPercentageAuto::length(t);
    self
  }
  fn mb(mut self, b: f32) -> Self {
    self.style().margin.bottom = taffy::LengthPercentageAuto::length(b);
    self
  }

  fn p(mut self, value: f32) -> Self {
    self.style().padding = taffy::Rect::length(value);
    self
  }
  fn px(mut self, x: f32) -> Self {
    self.style().padding.left = taffy::LengthPercentage::length(x);
    self.style().padding.right = taffy::LengthPercentage::length(x);
    self
  }
  fn py(mut self, y: f32) -> Self {
    self.style().padding.bottom = taffy::LengthPercentage::length(y);
    self.style().padding.top = taffy::LengthPercentage::length(y);
    self
  }
  fn pl(mut self, l: f32) -> Self {
    self.style().padding.left = taffy::LengthPercentage::length(l);
    self
  }
  fn pr(mut self, r: f32) -> Self {
    self.style().padding.right = taffy::LengthPercentage::length(r);
    self
  }
  fn pt(mut self, t: f32) -> Self {
    self.style().padding.top = taffy::LengthPercentage::length(t);
    self
  }
  fn pb(mut self, b: f32) -> Self {
    self.style().padding.bottom = taffy::LengthPercentage::length(b);
    self
  }

  fn border(mut self, value: f32) -> Self {
    self.style().border = taffy::Rect::length(value);
    self
  }
  fn border_x(mut self, x: f32) -> Self {
    self.style().border.left = taffy::LengthPercentage::length(x);
    self.style().border.right = taffy::LengthPercentage::length(x);
    self
  }
  fn border_y(mut self, y: f32) -> Self {
    self.style().border.bottom = taffy::LengthPercentage::length(y);
    self.style().border.top = taffy::LengthPercentage::length(y);
    self
  }
  fn border_l(mut self, l: f32) -> Self {
    self.style().border.left = taffy::LengthPercentage::length(l);
    self
  }
  fn border_r(mut self, r: f32) -> Self {
    self.style().border.right = taffy::LengthPercentage::length(r);
    self
  }
  fn border_t(mut self, t: f32) -> Self {
    self.style().border.top = taffy::LengthPercentage::length(t);
    self
  }
  fn border_b(mut self, b: f32) -> Self {
    self.style().border.bottom = taffy::LengthPercentage::length(b);
    self
  }

  fn items_start(mut self) -> Self {
    self.style().align_items = Some(taffy::AlignItems::FLEX_START);
    self
  }
  fn items_end(mut self) -> Self {
    self.style().align_items = Some(taffy::AlignItems::FLEX_END);
    self
  }
  fn items_end_safe(mut self) -> Self {
    self.style().align_items = Some(taffy::AlignItems::SAFE_FLEX_END);
    self
  }
  fn items_center(mut self) -> Self {
    self.style().align_items = Some(taffy::AlignItems::CENTER);
    self
  }
  fn items_center_safe(mut self) -> Self {
    self.style().align_items = Some(taffy::AlignItems::SAFE_CENTER);
    self
  }
  fn items_baseline(mut self) -> Self {
    self.style().align_items = Some(taffy::AlignItems::BASELINE);
    self
  }
  fn items_stretch(mut self) -> Self {
    self.style().align_items = Some(taffy::AlignItems::STRETCH);
    self
  }

  fn self_auto(mut self) -> Self {
    self.style().align_self = self.style().align_items;
    self
  }
  fn self_start(mut self) -> Self {
    self.style().align_self = Some(taffy::AlignItems::FLEX_START);
    self
  }
  fn self_end(mut self) -> Self {
    self.style().align_self = Some(taffy::AlignItems::FLEX_END);
    self
  }
  fn self_end_safe(mut self) -> Self {
    self.style().align_self = Some(taffy::AlignItems::SAFE_FLEX_END);
    self
  }
  fn self_center(mut self) -> Self {
    self.style().align_self = Some(taffy::AlignItems::CENTER);
    self
  }
  fn self_center_safe(mut self) -> Self {
    self.style().align_self = Some(taffy::AlignItems::SAFE_CENTER);
    self
  }
  fn self_baseline(mut self) -> Self {
    self.style().align_self = Some(taffy::AlignItems::BASELINE);
    self
  }
  fn self_stretch(mut self) -> Self {
    self.style().align_self = Some(taffy::AlignItems::STRETCH);
    self
  }

  fn justify_items_start(mut self) -> Self {
    self.style().justify_items = Some(taffy::AlignItems::START);
    self
  }
  fn justify_items_end(mut self) -> Self {
    self.style().justify_items = Some(taffy::AlignItems::END);
    self
  }
  fn justify_items_end_safe(mut self) -> Self {
    self.style().justify_items = Some(taffy::AlignItems::SAFE_END);
    self
  }
  fn justify_items_center(mut self) -> Self {
    self.style().justify_items = Some(taffy::AlignItems::CENTER);
    self
  }
  fn justify_items_center_safe(mut self) -> Self {
    self.style().justify_items = Some(taffy::AlignItems::SAFE_CENTER);
    self
  }
  fn justify_items_stretch(mut self) -> Self {
    self.style().justify_items = Some(taffy::AlignItems::STRETCH);
    self
  }

  fn justify_self_auto(mut self) -> Self {
    self.style().justify_self = self.style().justify_self;
    self
  }
  fn justify_self_start(mut self) -> Self {
    self.style().justify_self = Some(taffy::AlignItems::START);
    self
  }
  fn justify_self_end(mut self) -> Self {
    self.style().justify_self = Some(taffy::AlignItems::END);
    self
  }
  fn justify_self_end_safe(mut self) -> Self {
    self.style().justify_self = Some(taffy::AlignItems::SAFE_END);
    self
  }
  fn justify_self_center(mut self) -> Self {
    self.style().justify_self = Some(taffy::AlignItems::CENTER);
    self
  }
  fn justify_self_center_safe(mut self) -> Self {
    self.style().justify_self = Some(taffy::AlignItems::SAFE_CENTER);
    self
  }
  fn justify_self_stretch(mut self) -> Self {
    self.style().justify_self = Some(taffy::AlignItems::STRETCH);
    self
  }

  fn content_start(mut self) -> Self {
    self.style().align_content = Some(taffy::AlignContent::FLEX_START);
    self
  }
  fn content_end(mut self) -> Self {
    self.style().align_content = Some(taffy::AlignContent::FLEX_END);
    self
  }
  fn content_center(mut self) -> Self {
    self.style().align_content = Some(taffy::AlignContent::CENTER);
    self
  }
  fn content_between(mut self) -> Self {
    self.style().align_content = Some(taffy::AlignContent::SPACE_BETWEEN);
    self
  }
  fn content_around(mut self) -> Self {
    self.style().align_content = Some(taffy::AlignContent::SPACE_AROUND);
    self
  }
  fn content_evenly(mut self) -> Self {
    self.style().align_content = Some(taffy::AlignContent::SPACE_EVENLY);
    self
  }
  fn content_stretch(mut self) -> Self {
    self.style().align_content = Some(taffy::AlignContent::STRETCH);
    self
  }

  fn justify_start(mut self) -> Self {
    self.style().justify_content = Some(taffy::AlignContent::FLEX_START);
    self
  }
  fn justify_end(mut self) -> Self {
    self.style().justify_content = Some(taffy::AlignContent::FLEX_END);
    self
  }
  fn justify_end_safe(mut self) -> Self {
    self.style().justify_content = Some(taffy::AlignContent::SAFE_FLEX_END);
    self
  }
  fn justify_center(mut self) -> Self {
    self.style().justify_content = Some(taffy::AlignContent::CENTER);
    self
  }
  fn justify_center_safe(mut self) -> Self {
    self.style().justify_content = Some(taffy::AlignContent::SAFE_CENTER);
    self
  }
  fn justify_between(mut self) -> Self {
    self.style().justify_content = Some(taffy::AlignContent::SPACE_BETWEEN);
    self
  }
  fn justify_around(mut self) -> Self {
    self.style().justify_content = Some(taffy::AlignContent::SPACE_AROUND);
    self
  }
  fn justify_evenly(mut self) -> Self {
    self.style().justify_content = Some(taffy::AlignContent::SPACE_EVENLY);
    self
  }
  fn justify_stretch(mut self) -> Self {
    self.style().justify_content = Some(taffy::AlignContent::STRETCH);
    self
  }

  fn gap(mut self, value: f32) -> Self {
    self.style().gap = taffy::Size::length(value);
    self
  }
  fn gap_x(mut self, x: f32) -> Self {
    self.style().gap.width = taffy::LengthPercentage::length(x);
    self
  }
  fn gap_y(mut self, y: f32) -> Self {
    self.style().gap.height = taffy::LengthPercentage::length(y);
    self
  }

  fn flex_row(mut self) -> Self {
    self.style().flex_direction = taffy::FlexDirection::Row;
    self
  }
  fn flex_row_reverse(mut self) -> Self {
    self.style().flex_direction = taffy::FlexDirection::RowReverse;
    self
  }
  fn flex_col(mut self) -> Self {
    self.style().flex_direction = taffy::FlexDirection::Column;
    self
  }
  fn flex_col_reverse(mut self) -> Self {
    self.style().flex_direction = taffy::FlexDirection::ColumnReverse;
    self
  }

  fn flex_nowrap(mut self) -> Self {
    self.style().flex_wrap = taffy::FlexWrap::NoWrap;
    self
  }
  fn flex_wrap(mut self) -> Self {
    self.style().flex_wrap = taffy::FlexWrap::Wrap;
    self
  }
  fn flex_wrap_reverse(mut self) -> Self {
    self.style().flex_wrap = taffy::FlexWrap::WrapReverse;
    self
  }

  fn flex_basis(mut self, value: f32) -> Self {
    self.style().flex_basis = taffy::Dimension::length(value);
    self
  }
  fn flex_grow(mut self, value: f32) -> Self {
    self.style().flex_grow = value;
    self
  }
  fn flex_shirnk(mut self, value: f32) -> Self {
    self.style().flex_shrink = value;
    self
  }
}
