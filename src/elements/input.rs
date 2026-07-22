// SPDX-License-Identifier: Apache-2.0

use std::{ops::Range, sync::Arc, time::Duration};

use ropey::Rope;
use smallvec::{SmallVec, smallvec};

use crate::{
  App, AppContext, Component, Context, Element, ElementExt, Entity,
  EventDispatcher, FocusHandle, Focusable, InputHandler, InteractiveElement,
  IntoElement, Keybind, Keystroke, ParentElement, RenderOnce, StyleableElement,
  Task, Window, div, get_terminal,
};

mod actions {
  use crate::actions;

  actions! {
    "input",
    [
      Delete,
      DeleteTillLineStart,
      DeleteTillLineEnd,
      DeleteTillWordStart,
      SelectAll,
      SelectTillLineStart,
      SelectTillLineEnd,
      SelectTillWordStart,
      SelectTillWordEnd,
      Copy,
      Cut,
      Paste,
      Undo,
      Redo,
      Left,
      Right,
      Up,
      Down,
      Home,
      End,
      Enter,
      Escape,
    ]
  }
}
use self::actions::*;

#[derive(Debug)]
pub struct Input {
  state: Entity<InputState>,
  style: taffy::Style,
  tab_index: isize,
  disabled: bool,
}
impl Input {
  pub fn new(state: &Entity<InputState>) -> Self {
    Self {
      state: state.clone(),
      style: taffy::Style::DEFAULT,
      tab_index: 0,
      disabled: false,
    }
  }

  pub fn tab_index(mut self, tab_index: isize) -> Self {
    self.tab_index = tab_index;
    self
  }
  pub fn disabled(mut self, disabled: bool) -> Self {
    self.disabled = disabled;
    self
  }
}
impl RenderOnce for Input {
  fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
    let state = self.state.read(cx);

    div()
      .track_focus(&state.focus_handle)
      .tab_index(self.tab_index)
      .when(!state.disabled, |this| {
        this
          .on_action(window.listener(&self.state, InputState::delete))
          .on_action(window.listener(&self.state, InputState::move_left))
          .on_action(window.listener(&self.state, InputState::move_right))
          .on_action(window.listener(&self.state, InputState::move_up))
          .on_action(window.listener(&self.state, InputState::move_down))
          .on_action(window.listener(&self.state, InputState::enter))
          .on_action(window.listener(&self.state, InputState::escape))
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
impl IntoElement for Input {
  type Element = Component<Self>;

  fn into_element(self) -> Self::Element {
    Component::new(self)
  }
}
impl StyleableElement for Input {
  fn style(&mut self) -> &mut taffy::Style {
    &mut self.style
  }
}

#[derive(Debug)]
pub struct InputState {
  focus_handle: FocusHandle,
  text: Rope,
  placeholder: Option<Arc<str>>,
  mode: InputMode,
  disabled: bool,
  cursor_pos: usize,
  selection: Option<Range<usize>>,
  cursor: Entity<CursorBlinking>,
}
impl InputState {
  pub fn new(cx: &mut Context<Self>) -> Self {
    let mut focus_handle = cx.focus_handle();
    focus_handle.tab_stop(true);
    let cursor_blinker = cx.new_entity(CursorBlinking::new);

    Self {
      focus_handle,
      text: Rope::new(),
      placeholder: None,
      mode: InputMode::SingleLine,
      disabled: false,
      cursor_pos: 0,
      selection: None,
      cursor: cursor_blinker,
    }
  }

  fn delete(&mut self, _: &Delete, _: &mut Window, _: &mut Context<Self>) {
    if self.text.len_chars() > 0 && self.cursor_pos > 0 {
      self.cursor_pos -= 1;
      self.text.remove(self.cursor_pos..=self.cursor_pos);
    };
  }
  fn move_left(&mut self, _: &Left, _: &mut Window, _: &mut Context<Self>) {
    if self.cursor_pos > 0 {
      self.cursor_pos -= 1;
    };
  }
  fn move_right(&mut self, _: &Right, _: &mut Window, _: &mut Context<Self>) {
    if self.cursor_pos < self.text.len_chars() {
      self.cursor_pos += 1;
    };
  }
  fn move_up(&mut self, _: &Up, _: &mut Window, _: &mut Context<Self>) {
    if matches!(self.mode, InputMode::SingleLine) {
      return;
    };

    todo!();
  }
  fn move_down(&mut self, _: &Down, _: &mut Window, _: &mut Context<Self>) {
    if matches!(self.mode, InputMode::SingleLine) {
      return;
    };

    todo!();
  }
  fn enter(&mut self, _: &Enter, _: &mut Window, cx: &mut Context<Self>) {
    if matches!(self.mode, InputMode::MultiLine) {
      todo!("insert new line");
    };

    cx.emit(InputEvent::Submit);
  }
  fn escape(&mut self, _: &Escape, _: &mut Window, _: &mut Context<Self>) {
    self.selection = None;
  }
}
impl InputState {
  pub fn text(&self) -> String {
    self.text.to_string()
  }
}
impl InputHandler for InputState {
  fn insert_str(
    &mut self,
    range: Option<Range<usize>>,
    str: &str,
    window: &mut Window,
    cx: &mut crate::App,
  ) {
    self.text.insert(self.cursor_pos, str);
    self.cursor_pos += 1;
  }
  fn selected_text(
    &mut self,
    window: &mut Window,
    cx: &mut crate::App,
  ) -> Option<(Range<usize>, bool)> {
    match self.selection.clone() {
      Some(range) => {
        let is_reversed = range.start > self.cursor_pos;
        Some((range, is_reversed))
      }
      None => None,
    }
  }
}
impl Focusable for InputState {
  fn focus_handle(&self, _: &App) -> FocusHandle {
    self.focus_handle.clone()
  }
}
impl EventDispatcher<InputEvent> for InputState {}

#[derive(Debug)]
pub enum InputMode {
  SingleLine,
  MultiLine,
}
#[derive(Debug)]
pub enum InputEvent {
  Submit,
  Change,
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

const CURSOR_BLINK_INTERVAL: Duration = Duration::from_millis(499);

#[derive(Debug)]
pub(crate) struct CursorBlinking {
  pub(crate) visible: bool,
  step: usize,
  _task: Option<Task<()>>,
}
impl CursorBlinking {
  fn new(cx: &mut Context<Self>) -> Self {
    let mut this = Self {
      visible: true,
      step: 0,
      _task: None,
    };
    this.start_blinking(cx);
    this
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

const KEY_CONTEXT: &str = "input";
pub(crate) fn init(cx: &mut App) {
  let key_context = Some(KEY_CONTEXT);
  cx.bind_keys([
    Keybind::new(Delete, [Keystroke::parse("delete")], key_context),
    Keybind::new(Left, [Keystroke::parse("left")], key_context),
    Keybind::new(Right, [Keystroke::parse("right")], key_context),
    Keybind::new(Up, [Keystroke::parse("up")], key_context),
    Keybind::new(Down, [Keystroke::parse("down")], key_context),
    Keybind::new(Enter, [Keystroke::parse("return")], key_context),
    Keybind::new(Escape, [Keystroke::parse("esc")], key_context),
  ]);
}
