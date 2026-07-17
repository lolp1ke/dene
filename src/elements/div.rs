// SPDX-License-Identifier: Apache-2.0

use smallvec::SmallVec;

use crate::{
  AnyElement, App, Element, InteractiveElement, Interactivity, IntoElement,
  ParentElement, Rect, StyleableElement, Window, get_terminal,
};

#[derive(Debug)]
#[derive(Default)]
pub struct Div {
  interactivity: Interactivity,
  children: Vec<AnyElement>,
}
impl Element for Div {
  type RequestLayoutState = SmallVec<[taffy::NodeId; 8]>;
  type PreRenderState = ();

  fn request_layout(
    &mut self,
    window: &mut Window,
    cx: &mut App,
  ) -> (taffy::NodeId, Self::RequestLayoutState) {
    if self.interactivity.focusable
      && self.interactivity.tracking_focus_handle.is_none()
    {
      self.interactivity.tracking_focus_handle = Some(cx.focus_handle());
    };

    let child_node_ids = self
      .children
      .iter_mut()
      .map(|child| child.request_layout(window, cx))
      .collect::<SmallVec<_>>();

    let node_id = window.request_layout(
      self.interactivity.base_style.clone(),
      &child_node_ids,
      cx,
    );
    (node_id, child_node_ids)
  }
  fn pre_render(
    &mut self,
    _: Rect,
    _: &mut Self::RequestLayoutState,
    window: &mut Window,
    cx: &mut App,
  ) -> Self::PreRenderState {
    if let Some(focus_handle) =
      self.interactivity.tracking_focus_handle.as_ref()
    {
      window.set_focus_handle(focus_handle);
    };

    if matches!(self.interactivity.base_style.display, taffy::Display::None) {
      return;
    };

    for child in self.children.iter_mut() {
      child.pre_render(window, cx);
    }
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

    let border = self.interactivity.base_style.border;
    draw_border(bounds, border);

    self.interactivity.apply_keyboard_listeners(window);
    for child in self.children.iter_mut() {
      child.render(window, cx);
    }
  }
}
impl IntoElement for Div {
  type Element = Self;

  fn into_element(self) -> Self::Element {
    self
  }
}
impl ParentElement for Div {
  fn child(mut self, child: impl IntoElement) -> Self {
    self.children.push(child.into_any_element());
    self
  }

  fn children<I>(mut self, children: I) -> Self
  where
    I: IntoIterator,
    I::Item: IntoElement,
  {
    self
      .children
      .extend(children.into_iter().map(|child| child.into_any_element()));
    self
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

  if (bl | br | bt | bb) == 0 {
    return;
  };
  let mut terminal = get_terminal().write();

  let left = bounds.x;
  let right = bounds.x + bounds.width - 1;
  let top = bounds.y;
  let bottom = bounds.y + bounds.height - 1;

  if bl > 0 {
    let y_start = top + bt;
    let y_end = bottom - bb;
    if y_start <= y_end {
      for y in y_start..=y_end {
        terminal.write_at(left, y, "│".as_bytes());
      }
    };
  };
  if br > 0 {
    let y_start = top + bt;
    let y_end = bottom - bb;
    if y_start <= y_end {
      for y in y_start..=y_end {
        terminal.write_at(right, y, "│".as_bytes());
      }
    };
  };
  if bt > 0 {
    let y = top;
    if bl > 0 {
      terminal.write_at(left, y, "┌".as_bytes());
    };
    let x_start = left + bl;
    let x_end = right - br;
    if x_start <= x_end {
      let line = "─".repeat((x_end - x_start + 1) as usize);
      terminal.write_at(x_start, y, line.as_bytes());
    };
    if br > 0 {
      terminal.write_at(right, y, "┐".as_bytes());
    };
  };
  if bb > 0 {
    let y = bottom;

    if bl > 0 {
      terminal.write_at(left, y, "└".as_bytes());
    };
    let x_start = left + bl;
    let x_end = right - br;
    if x_start <= x_end {
      let line = "─".repeat((x_end - x_start + 1) as usize);
      terminal.write_at(x_start, y, line.as_bytes());
    };
    if br > 0 {
      terminal.write_at(right, y, "┘".as_bytes());
    };
  };

  terminal.flush();
}
