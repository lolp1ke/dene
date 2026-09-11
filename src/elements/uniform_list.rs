// SPDX-License-Identifier: Apache-2.0

use std::{cell::RefCell, ops::Range, rc::Rc};

use smallvec::SmallVec;
use taffy::{AvailableSpace, Dimension};

pub use crate::geometry::Axis;
use crate::{
  AnyElement, App, Context, DispatchPhase, Div, Element, Entity, Hitbox,
  IntoElement, ParentElement, Rect, ScrollWheelEvent, StyleableElement, Window,
  div, get_terminal,
};

type RenderItemsFn = Box<
  dyn FnMut(Range<usize>, &mut Window, &mut App) -> SmallVec<[AnyElement; 16]>,
>;

#[derive(Clone, Copy, Debug, Default)]
pub enum ScrollAlignment {
  Start,
  Center,
  End,
  #[default]
  Nearest,
}

#[derive(Clone, Debug, Default)]
pub struct UniformListScrollHandle(Rc<RefCell<ScrollState>>);

#[derive(Debug, Default)]
struct ScrollState {
  offset: usize,
  item_extent: usize,
  viewport_extent: usize,
  content_extent: usize,
  visible_range: Range<usize>,
  pending_item: Option<(usize, ScrollAlignment)>,
}

impl UniformListScrollHandle {
  pub fn new() -> Self {
    Self::default()
  }

  pub fn offset(&self) -> usize {
    self.0.borrow().offset
  }

  pub fn set_offset(&self, offset: usize) {
    let mut state = self.0.borrow_mut();
    state.offset = offset;
    state.pending_item = None;
  }

  pub fn scroll_to_item(&self, index: usize, alignment: ScrollAlignment) {
    self.0.borrow_mut().pending_item = Some((index, alignment));
  }

  pub fn visible_range(&self) -> Range<usize> {
    self.0.borrow().visible_range.clone()
  }

  pub fn item_extent(&self) -> usize {
    self.0.borrow().item_extent
  }

  pub fn viewport_extent(&self) -> usize {
    self.0.borrow().viewport_extent
  }

  pub fn content_extent(&self) -> usize {
    self.0.borrow().content_extent
  }
}

impl ScrollState {
  fn update(&mut self, count: usize, item_extent: usize, viewport: usize) {
    self.item_extent = item_extent;
    self.viewport_extent = viewport;
    self.content_extent = count.saturating_mul(item_extent);
    if count == 0 {
      self.offset = 0;
      self.pending_item = None;
    }
    if viewport == 0 || item_extent == 0 || count == 0 {
      self.visible_range = 0..0;
      return;
    }

    let max_offset = self.content_extent.saturating_sub(viewport);
    self.offset = self.offset.min(max_offset);
    if let Some((index, alignment)) = self.pending_item.take() {
      let start = index.min(count - 1).saturating_mul(item_extent);
      let end = start.saturating_add(item_extent);
      self.offset = match alignment {
        ScrollAlignment::Start => start,
        ScrollAlignment::Center => start
          .saturating_add(item_extent / 2)
          .saturating_sub(viewport / 2),
        ScrollAlignment::End => end.saturating_sub(viewport),
        ScrollAlignment::Nearest => {
          if start < self.offset {
            start
          } else if end > self.offset.saturating_add(viewport) {
            start.min(end.saturating_sub(viewport))
          } else {
            self.offset
          }
        }
      }
      .min(max_offset);
    }
    let start = (self.offset / item_extent).min(count);
    let end = self
      .offset
      .saturating_add(viewport)
      .div_ceil(item_extent)
      .min(count);
    self.visible_range = start..end;
  }

  fn scroll(&mut self, delta: i32) -> bool {
    let distance = (delta.unsigned_abs() as usize).saturating_mul(3);
    let offset = if delta < 0 {
      self.offset.saturating_sub(distance)
    } else {
      self.offset.saturating_add(distance)
    }
    .min(self.content_extent.saturating_sub(self.viewport_extent));
    if offset == self.offset {
      return false;
    }
    self.offset = offset;
    self.pending_item = None;
    true
  }
}

#[derive(derive_more::Debug)]
pub struct UniformList {
  base: Div,
  axis: Axis,
  items_count: usize,
  measure_index: usize,
  scroll_handle: UniformListScrollHandle,
  #[debug(skip)]
  render_items: RenderItemsFn,
}

impl UniformList {
  pub fn new<F, I, R>(
    items_count: usize,
    scroll_handle: &UniformListScrollHandle,
    mut render_items: F,
  ) -> Self
  where
    F: 'static + FnMut(Range<usize>, &mut Window, &mut App) -> I,
    I: IntoIterator<Item = R>,
    R: IntoElement,
  {
    Self {
      base: div().size_full().min_w(0.).min_h(0.).overflow_hidden(),
      axis: Axis::Vertical,
      items_count,
      measure_index: 0,
      scroll_handle: scroll_handle.clone(),
      render_items: Box::new(move |range, window, cx| {
        render_items(range, window, cx)
          .into_iter()
          .map(IntoElement::into_any_element)
          .collect()
      }),
    }
  }

  pub fn axis(mut self, axis: Axis) -> Self {
    self.axis = axis;
    self
  }

  pub fn item_to_measure(mut self, index: usize) -> Self {
    self.measure_index = index;
    self
  }

  fn items(
    &mut self,
    range: Range<usize>,
    window: &mut Window,
    cx: &mut App,
  ) -> SmallVec<[AnyElement; 16]> {
    if range.is_empty() {
      return SmallVec::new();
    }
    let expected = range.len();
    let items = (self.render_items)(range, window, cx);
    assert_eq!(
      items.len(),
      expected,
      "UniformList renderer must return one element per requested index"
    );
    items
  }

  fn slot(&self, item: AnyElement, viewport: Rect) -> AnyElement {
    let mut slot = div().flex().child(item);
    match self.axis {
      Axis::Vertical => {
        slot = slot.flex_col();
        slot.style().size.width = Dimension::length(viewport.width as f32);
      }
      Axis::Horizontal => {
        slot = slot.flex_row();
        slot.style().size.height = Dimension::length(viewport.height as f32);
      }
    }
    slot.into_any_element()
  }
}

pub fn uniform_list<V, F, I, R>(
  view: &Entity<V>,
  items_count: usize,
  scroll_handle: &UniformListScrollHandle,
  mut render_items: F,
) -> UniformList
where
  V: 'static,
  F: 'static + FnMut(&mut V, Range<usize>, &mut Window, &mut Context<V>) -> I,
  I: IntoIterator<Item = R>,
  R: IntoElement,
{
  let view = view.clone();
  UniformList::new(items_count, scroll_handle, move |range, window, cx| {
    view.update(cx, |view, cx| render_items(view, range, window, cx))
  })
}

#[doc(hidden)]
pub struct UniformListLayoutState {
  node_id: taffy::NodeId,
  base: <Div as Element>::RequestLayoutState,
}

#[doc(hidden)]
pub struct UniformListPreRenderState {
  base: <Div as Element>::PreRenderState,
  viewport: Rect,
  items: SmallVec<[AnyElement; 16]>,
  item_extent: usize,
  remainder: usize,
}

impl Element for UniformList {
  type RequestLayoutState = UniformListLayoutState;
  type PreRenderState = UniformListPreRenderState;

  fn request_layout(
    &mut self,
    window: &mut Window,
    cx: &mut App,
  ) -> (taffy::NodeId, Self::RequestLayoutState) {
    self.base.style().overflow = taffy::Point {
      x: taffy::Overflow::Hidden,
      y: taffy::Overflow::Hidden,
    };
    let (node_id, base) = self.base.request_layout(window, cx);
    (node_id, UniformListLayoutState { node_id, base })
  }

  fn pre_render(
    &mut self,
    bounds: Rect,
    request_layout: &mut Self::RequestLayoutState,
    window: &mut Window,
    cx: &mut App,
  ) -> Self::PreRenderState {
    let base =
      self
        .base
        .pre_render(bounds, &mut request_layout.base, window, cx);
    let viewport = window.layout_engine.content_box(request_layout.node_id);
    let mut state = UniformListPreRenderState {
      base,
      viewport,
      items: SmallVec::new(),
      item_extent: 0,
      remainder: 0,
    };
    if self.items_count == 0 || viewport.width == 0 || viewport.height == 0 {
      let viewport_extent = match self.axis {
        Axis::Vertical => viewport.height,
        Axis::Horizontal => viewport.width,
      };
      self.scroll_handle.0.borrow_mut().update(
        self.items_count,
        0,
        viewport_extent as usize,
      );
      return state;
    }

    let (available, viewport_extent) = match self.axis {
      Axis::Vertical => (
        taffy::Size {
          width: AvailableSpace::Definite(viewport.width as f32),
          height: AvailableSpace::MaxContent,
        },
        viewport.height,
      ),
      Axis::Horizontal => (
        taffy::Size {
          width: AvailableSpace::MaxContent,
          height: AvailableSpace::Definite(viewport.height as f32),
        },
        viewport.width,
      ),
    };
    let measure_index =
      self.measure_index.min(self.items_count.saturating_sub(1));
    let input_handlers = std::mem::take(&mut window.next_frame.input_handlers);
    let item = self
      .items(measure_index..measure_index + 1, window, cx)
      .pop()
      .unwrap();
    let mut measured = self.slot(item, viewport);
    let measured_node = measured.request_layout(window, cx);
    let mut measured_handlers =
      std::mem::replace(&mut window.next_frame.input_handlers, input_handlers);
    window.layout_engine.compute(measured_node, available);
    let measured_bounds = window.layout_bounds(measured_node);
    let extent = match self.axis {
      Axis::Vertical => measured_bounds.height,
      Axis::Horizontal => measured_bounds.width,
    }
    .max(1) as usize;

    let range = {
      let mut scroll = self.scroll_handle.0.borrow_mut();
      scroll.update(self.items_count, extent, viewport_extent as usize);
      state.remainder = scroll.offset % extent;
      scroll.visible_range.clone()
    };
    state.item_extent = extent;
    let mut measured = Some((measured, measured_node));
    let includes_measurement = range.contains(&measure_index);
    let mut items = if includes_measurement {
      let mut items = self.items(range.start..measure_index, window, cx);
      items.extend(self.items(measure_index + 1..range.end, window, cx));
      items.into_iter()
    } else {
      self.items(range.clone(), window, cx).into_iter()
    };

    for index in range {
      let (mut item, node_id) = if index == measure_index {
        if !measured_handlers.is_empty() {
          window.next_frame.input_handlers =
            std::mem::take(&mut measured_handlers);
        }
        measured.take().unwrap()
      } else {
        let mut item = self.slot(items.next().unwrap(), viewport);
        let node_id = item.request_layout(window, cx);
        (item, node_id)
      };
      let mut style = window.layout_engine.style(node_id);
      match self.axis {
        Axis::Vertical => style.size.height = Dimension::length(extent as f32),
        Axis::Horizontal => style.size.width = Dimension::length(extent as f32),
      }
      window.layout_engine.set_style(node_id, style);
      window.layout_engine.compute(node_id, available);
      item.pre_render(window, cx);
      state.items.push(item);
    }
    state
  }

  fn render(
    &mut self,
    bounds: Rect,
    request_layout: &mut Self::RequestLayoutState,
    pre_render: &mut Self::PreRenderState,
    window: &mut Window,
    cx: &mut App,
  ) {
    self.base.render(
      bounds,
      &mut request_layout.base,
      &mut pre_render.base,
      window,
      cx,
    );
    if pre_render.items.is_empty() {
      return;
    }
    let viewport = Rect {
      x: bounds.x.saturating_add(pre_render.viewport.x),
      y: bounds.y.saturating_add(pre_render.viewport.y),
      ..pre_render.viewport
    };
    let hitbox = Hitbox {
      bounds: get_terminal().read().visible_bounds(viewport),
    };
    let handle = self.scroll_handle.clone();
    window.on_mouse_event(
      move |event: &ScrollWheelEvent, phase, window, cx| {
        if matches!(phase, DispatchPhase::Bubble)
          && hitbox.contains(event.pos)
          && handle.0.borrow_mut().scroll(event.scroll_delta)
        {
          window.dirty = true;
          cx.propagate_event = false;
        }
      },
    );

    let old_offset = {
      let mut terminal = get_terminal().write();
      terminal.push_clip(viewport);
      terminal.draw_offset
    };
    for (index, item) in pre_render.items.iter_mut().enumerate() {
      let displacement =
        (index * pre_render.item_extent) as i64 - pre_render.remainder as i64;
      {
        let mut terminal = get_terminal().write();
        terminal.draw_offset = (
          old_offset.0 + viewport.x as i64,
          old_offset.1 + viewport.y as i64,
        );
        let mut slot = Rect {
          x: 0,
          y: 0,
          ..pre_render.viewport
        };
        match self.axis {
          Axis::Vertical => {
            terminal.draw_offset.1 += displacement;
            slot.height = pre_render.item_extent as u16;
          }
          Axis::Horizontal => {
            terminal.draw_offset.0 += displacement;
            slot.width = pre_render.item_extent as u16;
          }
        }
        terminal.push_clip(slot);
      }
      item.render(window, cx);
      get_terminal().write().clip_rect_stack.pop();
    }
    let mut terminal = get_terminal().write();
    terminal.draw_offset = old_offset;
    terminal.clip_rect_stack.pop();
  }
}
impl IntoElement for UniformList {
  type Element = Self;

  fn into_element(self) -> Self {
    self
  }
}
impl StyleableElement for UniformList {
  fn style(&mut self) -> &mut taffy::Style {
    self.base.style()
  }
}
