// SPDX-License-Identifier: Apache-2.0

use std::{ops::Range, sync::Arc, time::Duration};

use ropey::Rope;

use crate::{
  App, AppContext, Context, ElementExt, Entity, FocusHandle, Focusable,
  InputHandler, InteractiveElement, IntoElement, Keybind, Keystroke,
  ParentElement, Render, StyleableElement, Task, Window, div,
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
      .child(div().border(1.).child(state.text.to_string()).when(
        state.cursor.read(cx).visible && state.focus_handle.is_focused(window),
        |this| this.child(div().child(state.cursor_style())),
      ))
  }
}

#[derive(Debug)]
pub struct InputState {
  pub focus_handle: FocusHandle,
  pub text: Rope,
  pub placeholder: Option<Arc<str>>,
  pub mode: InputMode,
  pub disabled: bool,
  pub cursor_pos: usize,
  pub selection: Option<Range<usize>>,
  cursor: Entity<Cursor>,
}
impl InputState {
  fn delete(&mut self, _: &Delete, _: &mut Window, _: &mut App) {
    if self.text.len_chars() > 0 && self.cursor_pos > 0 {
      self.cursor_pos -= 1;
      self.text.remove(self.cursor_pos..=self.cursor_pos);
    };
  }
  fn move_left(&mut self, _: &Left, _: &mut Window, _: &mut App) {
    if self.cursor_pos > 0 {
      self.cursor_pos -= 1;
    };
  }
  fn move_right(&mut self, _: &Right, _: &mut Window, _: &mut App) {
    if self.cursor_pos < self.text.len_chars() {
      self.cursor_pos += 1;
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

  fn cursor_style(&self) -> &str {
    // match self.cursor.style {
    //   CursorStyle::Bar => "▏",
    //   CursorStyle::Block => "█",
    //   CursorStyle::Underscore => "_",
    // }
    "▏"
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
      state.text.insert(state.cursor_pos, str);
      state.cursor_pos += 1;
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
        let is_reversed = range.start > state.cursor_pos;
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
      cursor_pos: 0,
      selection: None,
      cursor: cx.new_entity(|cx| {
        let mut cursor = Cursor::new();
        cursor.start_blinking(cx);
        cursor
      }),
    }),
  })
}

#[derive(Debug)]
pub enum InputMode {
  SingleLine,
  MultiLine,
}

const CURSOR_BLINK_INTERVAL: Duration = Duration::from_millis(499);

#[derive(Debug)]
pub(crate) struct Cursor {
  pub(crate) style: CursorStyle,
  pub(crate) visible: bool,
  step: usize,
  _task: Option<Task<()>>,
}
impl Cursor {
  fn new() -> Self {
    Self {
      style: CursorStyle::Bar,
      visible: true,
      step: 0,
      _task: None,
    }
  }

  fn start_blinking(&mut self, cx: &mut Context<Self>) {
    let next_step = self.step + 1;
    self.step = next_step;
    self._task = Some(cx.spawn(async move |this, cx| {
      tokio::time::sleep(CURSOR_BLINK_INTERVAL).await;
      this.update(cx, |this, cx| {
        this.blink(next_step, cx);
      });
    }));
  }
  fn blink(&mut self, step: usize, cx: &mut Context<Self>) {
    if step != self.step {
      self.visible = true;
      return;
    };

    self.visible = !self.visible;
    cx.notify();

    self.step += 1;
    let next_step = self.step;
    self._task = Some(cx.spawn(async move |this, cx| {
      tokio::time::sleep(CURSOR_BLINK_INTERVAL).await;
      this.update(cx, |this, cx| {
        this.blink(next_step, cx);
      });
    }));
  }
}
#[derive(Debug)]
pub enum CursorStyle {
  Bar,
  Block,
  Underscore,
}
