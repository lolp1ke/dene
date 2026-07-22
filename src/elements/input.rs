// SPDX-License-Identifier: Apache-2.0

use std::{ops::Range, sync::Arc, time::Duration};

use ropey::Rope;
use smallvec::{SmallVec, smallvec};

use crate::{
  App, AppContext, Context, Element, ElementExt, Entity, FocusHandle,
  Focusable, InputHandler, InteractiveElement, IntoElement, Keybind, Keystroke,
  ParentElement, Render, StyleableElement, Task, Window, div, get_terminal,
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
      .child(
        div()
          .border(1.)
          .min_w(32.)
          .min_h(3.)
          .max_w(32.)
          .max_h(3.)
          .child(InputContent {
            text: state.text.to_string(),
            cursors: smallvec![Cursor {
              pos: state.cursor_pos,
              visible: state.cursor.read(cx).visible
                && state.focus_handle.is_focused(window),
              style: CursorStyle::Underscore,
            }],
          }),
      )
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
  cursor: Entity<CursorBlinking>,
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
        let mut cursor = CursorBlinking::new();
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
pub(crate) struct CursorBlinking {
  pub(crate) visible: bool,
  step: usize,
  _task: Option<Task<()>>,
}
impl CursorBlinking {
  fn new() -> Self {
    Self {
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

#[derive(Debug)]
pub struct InputContent {
  pub(crate) text: String,
  cursors: SmallVec<[Cursor; 2]>,
}
impl Element for InputContent {
  type RequestLayoutState = ();
  type PreRenderState = ();

  fn request_layout(
    &mut self,
    window: &mut Window,
    cx: &mut App,
  ) -> (taffy::NodeId, Self::RequestLayoutState) {
    let width = self.text.len() as f32;
    let height = self.text.lines().count() as f32;
    let mut style = taffy::Style::DEFAULT;
    style.min_size = taffy::Size {
      width: taffy::Dimension::length(width),
      height: taffy::Dimension::length(height.max(1.)),
    };
    style.size = taffy::Size {
      width: taffy::Dimension::length(width),
      height: taffy::Dimension::length(height),
    };
    let node_id = window.request_layout(style, &[], cx);
    (node_id, ())
  }
  fn pre_render(
    &mut self,
    _: crate::Rect,
    _: &mut Self::RequestLayoutState,
    _: &mut Window,
    _: &mut App,
  ) -> Self::PreRenderState {
  }
  fn render(
    &mut self,
    bounds: crate::Rect,
    _: &mut Self::RequestLayoutState,
    _: &mut Self::PreRenderState,
    _: &mut Window,
    _: &mut App,
  ) {
    let mut terminal = get_terminal().write();

    for (i, line) in self.text.lines().enumerate() {
      let y = bounds.y + i as u16;
      if y >= bounds.y + bounds.height {
        break;
      };

      terminal.write_at(bounds.x, y, line);
    }

    for cursor in self.cursors.iter() {
      if !cursor.visible {
        continue;
      };

      let x = bounds.x + cursor.pos as u16;
      let y = bounds.y;

      match cursor.style {
        CursorStyle::Bar => {
          let original = self.text[cursor.pos..].chars().next().unwrap_or(' ');
          let mut buf = [0u8; 4];
          let orig = original.encode_utf8(&mut buf);
          terminal.write_ansi_at(x, y, "\u{258f}", orig);
        }
        CursorStyle::Block => {
          if let Some(ch) = self.text[cursor.pos..].chars().next() {
            let mut buf = [0; 4];
            let buf = ch.encode_utf8(&mut buf);
            terminal.write_ansi_at(
              x,
              y,
              &format!("\x1b[7m{}\x1b[27m", buf),
              buf,
            );
          } else {
            terminal.write_ansi_at(x, y, "\x1b[7m \x1b[27m", " ");
          };
        }
        CursorStyle::Underscore => {
          if let Some(ch) = self.text[cursor.pos..].chars().next() {
            let mut buf = [0; 4];
            let buf = ch.encode_utf8(&mut buf);
            terminal.write_ansi_at(
              x,
              y,
              &format!("\x1b[4m{}\x1b[24m", buf),
              buf,
            );
          } else {
            terminal.write_ansi_at(x, y, "\x1b[4m \x1b[24m", " ");
          };
        }
      };
    }
  }
}
impl IntoElement for InputContent {
  type Element = Self;

  fn into_element(self) -> Self::Element {
    self
  }
}
#[derive(Debug)]
struct Cursor {
  pub(crate) pos: usize,
  pub(crate) visible: bool,
  pub(crate) style: CursorStyle,
}
