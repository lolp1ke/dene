// SPDX-License-Identifier: Apache-2.0

use std::{ops::Range, os::macos::raw::stat, sync::Arc};

use ropey::Rope;

use crate::{
  App, Context, ElementExt, Entity, FocusHandle, InputHandler,
  InteractiveElement, IntoElement, Render, Window, actions, div,
};

actions! {
  "input",
  [
    Delete,
    Left,
    Right,
    Up,
    Down,
    Home,
    End,
  ]
}

#[derive(Debug)]
pub struct Input {
  pub state: Entity<InputState>,
}
impl Render for Input {
  fn render(
    &mut self,
    window: &mut Window,
    cx: &mut Context<Self>,
  ) -> impl IntoElement {
    let state = self.state.read(cx);

    div()
      .track_focus(&state.focus_handle)
      .when(!state.disabled, |this| {
        this.on_action(window.listener(&self.state, InputState::delete))
      })
  }
}

#[derive(Debug)]
pub struct InputState {
  pub focus_handle: FocusHandle,
  pub text: Rope,
  pub placeholder: Option<Arc<str>>,
  pub disabled: bool,
}
impl InputState {
  fn delete(&mut self, _: &Delete, window: &mut Window, cx: &mut App) {
    panic!("action fires, :)");
  }
}
impl InputHandler for Input {
  fn insert_str(
    &mut self,
    range: Option<Range<usize>>,
    str: &str,
    window: &mut Window,
    cx: &mut crate::App,
  ) {
    self.state.update(cx, |state, cx| {
      state.text.insert(0, str);
    });
  }
  fn selected_text(
    &mut self,
    window: &mut Window,
    cx: &mut crate::App,
  ) -> Option<(Range<usize>, bool)> {
    None
  }
}
