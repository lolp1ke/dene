// SPDX-License-Identifier: Apache-2.0

use smallvec::SmallVec;

use crate::{
  AnyElement, App, Element, Hitbox, InteractiveElement, Interactivity,
  IntoElement, ParentElement, Pos, Rect, Size, StyleableElement, Window,
  apply_style, get_terminal,
};

#[derive(Debug)]
#[derive(Default)]
pub struct Div {
  interactivity: Interactivity,
  children: Vec<AnyElement>,
}
impl Element for Div {
  type RequestLayoutState = SmallVec<[taffy::NodeId; 8]>;
  type PreRenderState = Option<Hitbox>;

  fn request_layout(
    &mut self,
    window: &mut Window,
    cx: &mut App,
  ) -> (taffy::NodeId, Self::RequestLayoutState) {
    // let mut final_style = self.interactivity.base_style.clone();
    // apply_style(&mut final_style, self.interactivity.base_style.clone());

    if self.interactivity.focusable
      && self.interactivity.tracking_focus_handle.is_none()
    {
      let mut focus_handle = cx.focus_handle();
      if let Some(tab_index) = self.interactivity.tab_index {
        focus_handle.tab_index(tab_index);
      };
      focus_handle.tab_stop(self.interactivity.tab_stop);
      self.interactivity.tracking_focus_handle = Some(focus_handle);
    };

    if let Some(scroll_handle) =
      self.interactivity.tracking_scroll_handle.as_ref()
    {
      self.interactivity.scroll_offset =
        Some(scroll_handle.0.borrow().offset.clone());
    } else if matches!(
      self.interactivity.base_style.overflow.x,
      taffy::Overflow::Scroll
    ) || matches!(
      self.interactivity.base_style.overflow.y,
      taffy::Overflow::Scroll
    ) {
      todo!();
    };

    if let Some(focus_style) = self.interactivity.focus_style.as_ref()
      && window.focused(cx).is_some()
    {
      // apply_style(&mut final_style, focus_style.clone());
      // final_style = focus_style.clone();
    };

    let child_node_ids = self
      .children
      .iter_mut()
      .map(|child| child.request_layout(window, cx))
      .collect::<SmallVec<_>>();

    let node_id = window.request_layout(
      self.interactivity.base_style.clone(),
      // final_style,
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
    if let Some(focus_handle) =
      self.interactivity.tracking_focus_handle.as_ref()
    {
      window.set_focus_handle(focus_handle);
    };
    if !self.interactivity.key_contexts.is_empty() {
      window.set_key_contexts(&self.interactivity.key_contexts);
    };

    let mut child_min = Pos {
      x: u16::MAX,
      y: u16::MAX,
    };
    let mut child_max = Pos::default();
    let _content_size = if request_layout.is_empty() {
      bounds.as_size()
    } else if let Some(scroll_handle) =
      self.interactivity.tracking_scroll_handle.as_ref()
    {
      for child_node_id in request_layout.iter() {
        let child_bounds = window.layout_bounds(*child_node_id);
        child_min = child_min.min(child_bounds.as_pos());
        child_max = child_max.max(child_bounds.as_bottom_right_pos());
      }
      let border = self.interactivity.base_style.border;
      let bl = border.left.into_raw().value() as u16;
      let br = border.right.into_raw().value() as u16;
      let bt = border.top.into_raw().value() as u16;
      let bb = border.bottom.into_raw().value() as u16;
      let mut lock = scroll_handle.0.borrow_mut();
      lock.bounds = Rect {
        x: bounds.x + bl,
        y: bounds.y + bt,
        width: bounds.width.saturating_sub(bl + br),
        height: bounds.height.saturating_sub(bt + bb),
      };
      drop(lock);
      let content_size = Size {
        width: child_max.x.saturating_sub(child_min.x),
        height: child_max.y.saturating_sub(child_min.y),
      };
      scroll_handle.0.borrow_mut().content_size = content_size;
      content_size
    } else {
      for child_node_id in request_layout.iter() {
        let child_bounds = window.layout_bounds(*child_node_id);
        child_min = child_min.min(child_bounds.as_pos());
        child_max = child_max.max(child_bounds.as_bottom_right_pos());
      }
      Size {
        width: child_max.x.saturating_sub(child_min.x),
        height: child_max.y.saturating_sub(child_min.y),
      }
    };

    if matches!(self.interactivity.base_style.display, taffy::Display::None) {
      return None;
    };

    for child in self.children.iter_mut() {
      child.pre_render(window, cx);
    }

    None
  }
  fn render(
    &mut self,
    bounds: Rect,
    _: &mut Self::RequestLayoutState,
    _: &mut Self::PreRenderState,
    window: &mut Window,
    cx: &mut App,
  ) {
    if matches!(self.interactivity.base_style.display, taffy::Display::None) {
      return;
    };

    let mut tab_index = None;
    if self.interactivity.tab_stop {
      tab_index = self.interactivity.tab_index;
    };
    if let Some(focus_handle) =
      self.interactivity.tracking_focus_handle.as_mut()
    {
      focus_handle.tab_index(self.interactivity.tab_index.unwrap_or(0));
      focus_handle.tab_stop(self.interactivity.tab_stop);
      window.next_frame.tab_stop_map.insert(focus_handle);
    };

    let scroll_offset = self
      .interactivity
      .scroll_offset
      .as_ref()
      .map(|pos| *pos.borrow())
      .unwrap_or_default();
    let border = self.interactivity.base_style.border;
    let bt = border.top.into_raw().value() as u16;
    let bb = border.bottom.into_raw().value() as u16;
    let bl = border.left.into_raw().value() as u16;
    let br = border.right.into_raw().value() as u16;
    let has_border = (bl | br | bt | bb) > 0;
    let overflow = self.interactivity.base_style.overflow;
    let clip_x = has_border
      || matches!(
        overflow.x,
        taffy::Overflow::Hidden
          | taffy::Overflow::Clip
          | taffy::Overflow::Scroll
      );
    let clip_y = has_border
      || matches!(
        overflow.y,
        taffy::Overflow::Hidden
          | taffy::Overflow::Clip
          | taffy::Overflow::Scroll
      );
    let hitbox = Hitbox {
      bounds: get_terminal().read().visible_bounds(bounds),
    };
    let has_clip = clip_x || clip_y;
    if has_clip {
      let clip = Rect {
        x: bounds.x.saturating_add(bl),
        y: bounds.y.saturating_add(bt),
        width: bounds.width.saturating_sub(bl.saturating_add(br)),
        height: bounds.height.saturating_sub(bt.saturating_add(bb)),
      };
      get_terminal().write().push_clip_axes(clip, clip_x, clip_y);
    };

    window.with_tab_group(tab_index, |window| {
      self.interactivity.apply_mouse_listeners(&hitbox, window);
      self.interactivity.apply_keyboard_listeners(window);
      let old_offset = {
        let mut terminal = get_terminal().write();
        let old_offset = terminal.draw_offset;
        terminal.draw_offset.0 -= i64::from(scroll_offset.x);
        terminal.draw_offset.1 -= i64::from(scroll_offset.y);
        old_offset
      };
      for child in self.children.iter_mut() {
        child.render(window, cx);
      }
      get_terminal().write().draw_offset = old_offset;
    });

    if has_clip {
      get_terminal().write().clip_rect_stack.pop();
    };
    let border = self.interactivity.base_style.border;
    draw_border(bounds, border);
  }
}
impl IntoElement for Div {
  type Element = Self;

  fn into_element(self) -> Self::Element {
    self
  }
}
impl ParentElement for Div {
  fn extend<Iter>(&mut self, children: Iter)
  where
    Iter: IntoIterator,
    Iter::Item: IntoElement,
  {
    self
      .children
      .extend(children.into_iter().map(IntoElement::into_any_element));
  }
}
impl StyleableElement for Div {
  fn style(&mut self) -> &mut taffy::Style {
    &mut self.interactivity.base_style
  }
}
impl InteractiveElement for Div {
  fn interactivity(&mut self) -> &mut Interactivity {
    &mut self.interactivity
  }
}

pub fn div() -> Div {
  Default::default()
}

fn draw_border(bounds: Rect, border: taffy::Rect<taffy::LengthPercentage>) {
  let bl = border.left.into_raw().value() as u16;
  let br = border.right.into_raw().value() as u16;
  let bt = border.top.into_raw().value() as u16;
  let bb = border.bottom.into_raw().value() as u16;

  if (bl | br | bt | bb) == 0 || bounds.width == 0 || bounds.height == 0 {
    return;
  };
  let mut terminal = get_terminal().write();

  let left = bounds.x;
  let right = bounds.x.saturating_add(bounds.width - 1);
  let top = bounds.y;
  let bottom = bounds.y.saturating_add(bounds.height - 1);

  if bl > 0 {
    for offset in bt..bounds.height.saturating_sub(bb) {
      if let Some(y) = top.checked_add(offset) {
        terminal.write_at(left, y, "│");
      }
    }
  };
  if br > 0 {
    for offset in bt..bounds.height.saturating_sub(bb) {
      if let Some(y) = top.checked_add(offset) {
        terminal.write_at(right, y, "│");
      }
    }
  };
  if bt > 0 {
    let y = top;
    if bl > 0 {
      terminal.write_at(left, y, "┌");
    };
    if let Some(x_start) = left.checked_add(bl) {
      let line =
        "─".repeat(bounds.width.saturating_sub(bl.saturating_add(br)) as usize);
      terminal.write_at(x_start, y, line.as_str());
    };
    if br > 0 {
      terminal.write_at(right, y, "┐");
    };
  };
  if bb > 0 {
    let y = bottom;

    if bl > 0 {
      terminal.write_at(left, y, "└");
    };
    if let Some(x_start) = left.checked_add(bl) {
      let line =
        "─".repeat(bounds.width.saturating_sub(bl.saturating_add(br)) as usize);
      terminal.write_at(x_start, y, line.as_str());
    };
    if br > 0 {
      terminal.write_at(right, y, "┘");
    };
  };
}
