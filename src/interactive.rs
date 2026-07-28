// SPDX-License-Identifier: Apache-2.0

use std::{any::Any, ops::Range};

use crate::{App, Entity, Keystroke, Modifiers, Pos, Window};

pub(crate) trait InputEvent: 'static {
  fn to_dene_input(self) -> DeneInput;
}
pub(crate) trait KeyboardEvent: InputEvent {}
pub(crate) trait MouseEvent: InputEvent {}

#[derive(Debug)]
pub(crate) enum DeneInput {
  MouseButtonDown(MouseButtonDownEvent),
  MouseButtonUp(MouseButtonUpEvent),
  MouseMove(Pos),
  ScrollDown(ScrollEvent),
  ScrollUp(ScrollEvent),
  KeyDown(KeyDownEvent),
  KeyUp(KeyUpEvent),
}
impl DeneInput {
  pub(crate) fn keyboard_event(&self) -> Option<&dyn Any> {
    match self {
      Self::KeyDown(event) => Some(event),
      Self::KeyUp(event) => Some(event),
      _ => None,
    }
  }
  pub(crate) fn mouse_event(&self) -> Option<&dyn Any> {
    match self {
      Self::MouseButtonDown(event) => Some(event),
      Self::MouseButtonUp(event) => Some(event),
      Self::MouseMove(event) => Some(event),
      Self::ScrollDown(event) | Self::ScrollUp(event) => Some(event),
      _ => None,
    }
  }
}

#[derive(Debug)]
#[derive(Clone, Copy)]
#[derive(PartialEq)]
#[derive(Default)]
pub enum MouseButton {
  #[default]
  Left,
  Middle,
  Right,
}
impl From<crossterm::event::MouseButton> for MouseButton {
  fn from(value: crossterm::event::MouseButton) -> Self {
    match value {
      crossterm::event::MouseButton::Left => Self::Left,
      crossterm::event::MouseButton::Middle => Self::Middle,
      crossterm::event::MouseButton::Right => Self::Right,
    }
  }
}

#[derive(Debug)]
#[derive(Clone)]
pub struct MouseButtonDownEvent {
  pub(crate) button: MouseButton,
  pub(crate) pos: Pos,
  pub(crate) modifiers: Modifiers,
}
impl InputEvent for MouseButtonDownEvent {
  fn to_dene_input(self) -> DeneInput {
    DeneInput::MouseButtonDown(self)
  }
}
impl MouseEvent for MouseButtonDownEvent {}
#[derive(Debug)]
#[derive(Clone)]
pub struct MouseButtonUpEvent {
  pub(crate) button: MouseButton,
  pub(crate) pos: Pos,
  pub(crate) modifiers: Modifiers,
}
impl InputEvent for MouseButtonUpEvent {
  fn to_dene_input(self) -> DeneInput {
    DeneInput::MouseButtonUp(self)
  }
}
impl MouseEvent for MouseButtonUpEvent {}

#[derive(Debug)]
#[derive(Clone)]
pub struct ScrollEvent {
  pub(crate) pos: Pos,
  pub(crate) modifiers: Modifiers,
}

#[derive(Debug)]
pub struct KeyDownEvent {
  pub(crate) keystroke: Keystroke,
  pub(crate) is_held: bool,
}
impl InputEvent for KeyDownEvent {
  fn to_dene_input(self) -> DeneInput {
    DeneInput::KeyDown(self)
  }
}
impl KeyboardEvent for KeyDownEvent {}

#[derive(Debug)]
pub struct KeyUpEvent {
  pub(crate) keystroke: Keystroke,
}
impl InputEvent for KeyUpEvent {
  fn to_dene_input(self) -> DeneInput {
    DeneInput::KeyUp(self)
  }
}
impl KeyboardEvent for KeyUpEvent {}

pub trait InputHandler: 'static {
  fn insert_str(
    &mut self,
    range: Option<Range<usize>>,
    str: &str,
    window: &mut Window,
    cx: &mut App,
  );
  fn selected_text(
    &mut self,
    window: &mut Window,
    cx: &mut App,
  ) -> Option<(Range<usize>, bool)>;
}
impl<V> InputHandler for Entity<V>
where
  V: InputHandler,
{
  fn insert_str(
    &mut self,
    range: Option<Range<usize>>,
    str: &str,
    window: &mut Window,
    cx: &mut App,
  ) {
    self.update(cx, |this, cx| {
      this.insert_str(range, str, window, cx);
    });
  }
  fn selected_text(
    &mut self,
    window: &mut Window,
    cx: &mut App,
  ) -> Option<(Range<usize>, bool)> {
    self.update(cx, |this, cx| this.selected_text(window, cx))
  }
}
