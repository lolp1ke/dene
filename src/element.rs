// SPDX-License-Identifier: Apache-2.0

use std::{
  any::{Any, TypeId},
  fmt::Debug,
  mem,
};

use rustc_hash::FxHashMap;

use crate::{
  Action, App, Context, DispatchNodeId, DispatchPhase, FocusHandle,
  KeyDownEvent, KeyUpEvent, Rect, Window,
};

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
    request_layout: &mut Self::RequestLayoutState,
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
pub trait ParentElement: Element {
  fn child(self, child: impl IntoElement) -> Self;
  fn children<I>(self, children: I) -> Self
  where
    I: IntoIterator,
    I::Item: IntoElement;
}

#[derive(Debug)]
pub struct AnyElement(pub(crate) Box<dyn ElementObject>);
impl AnyElement {
  pub(crate) fn request_layout(
    &mut self,
    window: &mut Window,
    cx: &mut App,
  ) -> taffy::NodeId {
    self.0.request_layout(window, cx)
  }
  pub(crate) fn pre_render(&mut self, window: &mut Window, cx: &mut App) {
    self.0.pre_render(window, cx);
  }
  pub(crate) fn render(&mut self, window: &mut Window, cx: &mut App) {
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
    _: &mut Self::RequestLayoutState,
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
        let (node_id, request_layout) = self.element.request_layout(window, cx);

        self.phase = DrawableObjectPhase::RequestLayout {
          node_id,
          request_layout,
        };
        node_id
      }
      _ => panic!("MUST BE CALLED ONCE"),
    }
  }
  fn pre_render(&mut self, window: &mut Window, cx: &mut App) {
    match mem::take(&mut self.phase) {
      DrawableObjectPhase::RequestLayout {
        node_id,
        mut request_layout,
      } => {
        let bounds = window.layout_bounds(node_id);
        let dispatch_node_id = window.next_frame.dispatch_tree.push_node();
        let pre_render =
          self
            .element
            .pre_render(bounds, &mut request_layout, window, cx);
        window.next_frame.dispatch_tree.pop_node();

        self.phase = DrawableObjectPhase::PreRender {
          dispatch_node_id,
          bounds,
          request_layout,
          pre_render,
        };
      }
      _ => panic!("MUST BE CALLED AFTER `request_layout`"),
    }
  }
  fn render(
    &mut self,
    window: &mut Window,
    cx: &mut App,
  ) -> (E::RequestLayoutState, E::PreRenderState) {
    match mem::take(&mut self.phase) {
      DrawableObjectPhase::PreRender {
        dispatch_node_id,
        bounds,
        mut request_layout,
        mut pre_render,
      } => {
        window
          .next_frame
          .dispatch_tree
          .set_active_node(dispatch_node_id);
        self.element.render(
          bounds,
          &mut request_layout,
          &mut pre_render,
          window,
          cx,
        );

        self.phase = DrawableObjectPhase::Rendered;
        (request_layout, pre_render)
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
    dispatch_node_id: DispatchNodeId,
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

type KeyDownListener =
  Box<dyn 'static + Fn(&KeyDownEvent, DispatchPhase, &mut Window, &mut App)>;
type KeyUpListener =
  Box<dyn 'static + Fn(&KeyUpEvent, DispatchPhase, &mut Window, &mut App)>;
type ActionListener =
  Box<dyn 'static + Fn(&dyn Any, DispatchPhase, &mut Window, &mut App)>;

#[derive(derive_more::Debug)]
#[derive(Default)]
pub struct Interactivity {
  pub(crate) tracking_focus_handle: Option<FocusHandle>,
  pub(crate) focusable: bool,
  pub(crate) tab_index: Option<isize>,
  pub(crate) tab_stop: bool,

  #[debug(skip)]
  pub(crate) base_style: taffy::Style,

  #[debug(skip)]
  pub(crate) key_down_listener: Vec<KeyDownListener>,
  #[debug(skip)]
  pub(crate) key_up_listener: Vec<KeyUpListener>,
  #[debug(skip)]
  pub(crate) action_listeners: Vec<(TypeId, ActionListener)>,
}
impl Interactivity {
  pub(crate) fn apply_keyboard_listeners(&mut self, window: &mut Window) {
    let key_down_listeners = mem::take(&mut self.key_down_listener);
    let key_up_listeners = mem::take(&mut self.key_up_listener);
    let action_listeners = mem::take(&mut self.action_listeners);

    for listener in key_down_listeners.into_iter() {
      window.on_key_event(listener);
    }
    for listener in key_up_listeners.into_iter() {
      window.on_key_event(listener);
    }
    for (action_ty_id, listener) in action_listeners.into_iter() {
      window.on_action(action_ty_id, listener);
    }
  }

  fn on_key_down<F>(&mut self, listener: F)
  where
    F: 'static + Fn(&KeyDownEvent, &mut Window, &mut App),
  {
    self
      .key_down_listener
      .push(Box::new(move |event, phase, window, cx| {
        if matches!(phase, DispatchPhase::Bubble) {
          (listener)(event, window, cx);
        };
      }));
  }
  fn capture_key_down<F>(&mut self, listener: F)
  where
    F: 'static + Fn(&KeyDownEvent, &mut Window, &mut App),
  {
    self
      .key_down_listener
      .push(Box::new(move |event, phase, window, cx| {
        if matches!(phase, DispatchPhase::Capture) {
          (listener)(event, window, cx);
        };
      }));
  }
  fn on_key_up<F>(&mut self, listener: F)
  where
    F: 'static + Fn(&KeyUpEvent, &mut Window, &mut App),
  {
    self
      .key_up_listener
      .push(Box::new(move |event, phase, window, cx| {
        if matches!(phase, DispatchPhase::Bubble) {
          (listener)(event, window, cx);
        };
      }));
  }
  fn capture_key_up<F>(&mut self, listener: F)
  where
    F: 'static + Fn(&KeyUpEvent, &mut Window, &mut App),
  {
    self
      .key_up_listener
      .push(Box::new(move |event, phase, window, cx| {
        if matches!(phase, DispatchPhase::Capture) {
          (listener)(event, window, cx);
        };
      }));
  }
  fn on_action<F, A>(&mut self, listener: F)
  where
    A: Action,
    F: 'static + Fn(&A, &mut Window, &mut App),
  {
    self.action_listeners.push((
      TypeId::of::<A>(),
      Box::new(move |action, phase, window, cx| {
        if matches!(phase, DispatchPhase::Bubble)
          && let Some(action) = action.downcast_ref::<A>()
        {
          (listener)(action, window, cx);
        };
      }),
    ));
  }
}
pub trait InteractiveElement: Sized {
  fn interactivity(&mut self) -> &mut Interactivity;

  fn track_focus(mut self, focus_handle: &FocusHandle) -> Self {
    self.interactivity().tracking_focus_handle = Some(focus_handle.clone());
    self.interactivity().focusable = true;
    self
  }
  fn tab_index(mut self, tab_index: isize) -> Self {
    self.interactivity().focusable = true;
    self.interactivity().tab_index = Some(tab_index);
    self.interactivity().tab_stop = true;
    self
  }
  fn tab_stop(mut self, tab_stop: bool) -> Self {
    self.interactivity().tab_stop = tab_stop;
    self
  }

  fn on_key_down<F>(mut self, listener: F) -> Self
  where
    F: 'static + Fn(&KeyDownEvent, &mut Window, &mut App),
  {
    self.interactivity().on_key_down(listener);
    self
  }
  fn capture_key_down<F>(mut self, listener: F) -> Self
  where
    F: 'static + Fn(&KeyDownEvent, &mut Window, &mut App),
  {
    self.interactivity().capture_key_down(listener);
    self
  }
  fn on_key_up<F>(mut self, listener: F) -> Self
  where
    F: 'static + Fn(&KeyUpEvent, &mut Window, &mut App),
  {
    self.interactivity().on_key_up(listener);
    self
  }
  fn capture_key_up<F>(mut self, listener: F) -> Self
  where
    F: 'static + Fn(&KeyUpEvent, &mut Window, &mut App),
  {
    self.interactivity().capture_key_up(listener);
    self
  }

  fn on_action<F, A>(mut self, listener: F) -> Self
  where
    A: Action,
    F: 'static + Fn(&A, &mut Window, &mut App),
  {
    self.interactivity().on_action(listener);
    self
  }
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

  fn w_auto(mut self) -> Self {
    self.style().size.width = taffy::Dimension::auto();
    self
  }
  fn h_auto(mut self) -> Self {
    self.style().size.height = taffy::Dimension::auto();
    self
  }
  fn w_full(mut self) -> Self {
    self.style().size.width = taffy::Dimension::percent(1.);
    self
  }
  fn h_full(mut self) -> Self {
    self.style().size.height = taffy::Dimension::percent(1.);
    self
  }
  fn size_auto(mut self) -> Self {
    self.style().size = taffy::Size {
      width: taffy::Dimension::auto(),
      height: taffy::Dimension::auto(),
    };
    self
  }
  fn size_full(mut self) -> Self {
    self.style().size = taffy::Size {
      width: taffy::Dimension::percent(1.),
      height: taffy::Dimension::percent(1.),
    };
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

pub trait ElementExt {
  fn map<F, U>(self, f: F) -> U
  where
    Self: Sized,
    F: FnOnce(Self) -> U,
  {
    f(self)
  }

  fn when<F>(self, condition: bool, f: F) -> Self
  where
    Self: Sized,
    F: FnOnce(Self) -> Self,
  {
    self.map(|this| if condition { f(this) } else { this })
  }
}
impl<T> ElementExt for T where T: IntoElement {}
