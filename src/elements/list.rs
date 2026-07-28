// SPDX-License-Identifier: Apache-2.0

use crate::{
  App, Component, Context, Entity, EventDispatcher, FocusHandle, Focusable,
  InteractiveElement, IntoElement, Keybind, Keystroke, ParentElement, Render,
  RenderOnce, StyleableElement, Window, div,
};

mod actions {
  use crate::actions;

  actions! {
    "list",
    [
      Escape,
      Enter,
      Prev,
      Next,
    ]
  }
}
use self::actions::*;

#[derive(Debug)]
#[derive(Clone, Copy)]
pub enum ListEvent {
  Submit(usize),
  Select(usize),
  Cancel,
}

#[derive(Debug)]
pub struct List {
  state: Entity<ListState>,
  style: taffy::Style,
}
impl List {
  pub fn new(state: &Entity<ListState>) -> Self {
    Self {
      state: state.clone(),
      style: taffy::Style::DEFAULT,
    }
  }
}
impl RenderOnce for List {
  fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
    div().child(self.state.clone())
  }
}
impl IntoElement for List {
  type Element = Component<Self>;

  fn into_element(self) -> Self::Element {
    Component::new(self)
  }
}
impl StyleableElement for List {
  fn style(&mut self) -> &mut taffy::Style {
    &mut self.style
  }
}

#[derive(Debug)]
pub struct ListState {
  pub(crate) focus_handle: FocusHandle,
}
impl Render for ListState {
  fn render(
    &mut self,
    window: &mut Window,
    cx: &mut Context<Self>,
  ) -> impl IntoElement {
    div()
      .flex()
      .flex_col()
      .track_focus(&self.focus_handle)
      .child("list")
  }
}
impl Focusable for ListState {
  fn focus_handle(&self, _: &App) -> FocusHandle {
    self.focus_handle.clone()
  }
}
impl EventDispatcher<ListEvent> for ListState {}

const KEY_CONTEXT: &str = "list";
pub(crate) fn init(cx: &mut App) {
  let key_context = Some(KEY_CONTEXT);
  cx.bind_keys([
    Keybind::new(Escape, [Keystroke::parse("esc")], key_context),
    Keybind::new(Enter, [Keystroke::parse("return")], key_context),
    Keybind::new(Prev, [Keystroke::parse("left")], key_context),
    Keybind::new(Next, [Keystroke::parse("right")], key_context),
    Keybind::new(Next, [Keystroke::parse("up")], key_context),
    Keybind::new(Prev, [Keystroke::parse("down")], key_context),
  ]);
}
