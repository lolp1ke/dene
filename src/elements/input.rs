// SPDX-License-Identifier: Apache-2.0

use std::{ops::Range, sync::Arc};

use ropey::Rope;

use crate::{
  App, AppContext, Context, ElementExt, Entity, FocusHandle, Focusable,
  InputHandler, InteractiveElement, IntoElement, Keybind, Keystroke,
  ParentElement, Render, StyleableElement, Window, div,
};

mod actions {
  use crate::actions;

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
}
use self::actions::*;

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
      .tab_stop(true)
      .when(!state.disabled, |this| {
        this
          .on_action(window.listener(&self.state, InputState::delete))
          .on_action(window.listener(&self.state, InputState::move_left))
          .on_action(window.listener(&self.state, InputState::move_right))
          .on_action(window.listener(&self.state, InputState::move_up))
          .on_action(window.listener(&self.state, InputState::move_down))
      })
      .child(div().border(1.).child(state.text.to_string()))
  }
}

#[derive(Debug)]
pub struct InputState {
  pub focus_handle: FocusHandle,
  pub text: Rope,
  pub placeholder: Option<Arc<str>>,
  pub mode: InputMode,
  pub disabled: bool,
  pub cursor: usize,
  pub selection: Option<Range<usize>>,
}
impl InputState {
  fn delete(&mut self, _: &Delete, _: &mut Window, _: &mut App) {
    if self.text.len_chars() > 0 && self.cursor > 0 {
      self.cursor -= 1;
      self.text.remove(self.cursor..=self.cursor);
    };
  }
  fn move_left(&mut self, _: &Left, _: &mut Window, _: &mut App) {
    if self.cursor > 0 {
      self.cursor -= 1;
    };
  }
  fn move_right(&mut self, _: &Right, _: &mut Window, _: &mut App) {
    if self.cursor < self.text.len_chars() {
      self.cursor += 1;
    };
  }
  fn move_up(&mut self, _: &Up, _: &mut Window, _: &mut App) {
    if matches!(self.mode, InputMode::SingleLine) {
      return;
    };

    todo!();
  }
  fn move_down(&mut self, _: &Down, _: &mut Window, _: &mut App) {
    if matches!(self.mode, InputMode::SingleLine) {
      return;
    };

    todo!();
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
    self.state.update(cx, |state, _| {
      state.text.insert(state.cursor, str);
      state.cursor += 1;
    });
  }
  fn selected_text(
    &mut self,
    window: &mut Window,
    cx: &mut crate::App,
  ) -> Option<(Range<usize>, bool)> {
    let state = self.state.read(cx);
    match state.selection.clone() {
      Some(range) => {
        let is_reversed = range.start > state.cursor;
        Some((range, is_reversed))
      }
      None => None,
    }
  }
}
impl Focusable for Input {
  fn focus_handle(&self, cx: &App) -> FocusHandle {
    self.state.read(cx).focus_handle.clone()
  }
}
impl Focusable for InputState {
  fn focus_handle(&self, _: &App) -> FocusHandle {
    self.focus_handle.clone()
  }
}

pub fn input(cx: &mut App) -> Entity<Input> {
  // cx.bind_keys([Keybind {
  //   action: Box::new(Delete),
  //   keystrokes: smallvec![Keystroke::parse("").unwrap(),],
  // }]);
  let key_context = Some("input");
  cx.bind_keys([
    Keybind::new(Delete, [Keystroke::parse("delete")], key_context),
    Keybind::new(Left, [Keystroke::parse("left")], key_context),
    Keybind::new(Right, [Keystroke::parse("right")], key_context),
    Keybind::new(Up, [Keystroke::parse("up")], key_context),
    Keybind::new(Down, [Keystroke::parse("down")], key_context),
  ]);

  cx.new_entity(|cx| Input {
    state: cx.new_entity(|cx| InputState {
      focus_handle: cx.focus_handle(),
      text: Rope::new(),
      placeholder: None,
      mode: InputMode::SingleLine,
      disabled: false,
      cursor: 0,
      selection: None,
    }),
  })
}

#[derive(Debug)]
pub enum InputMode {
  SingleLine,
  MultiLine,
}
