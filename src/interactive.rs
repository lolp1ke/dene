// SPDX-License-Identifier: Apache-2.0

use std::any::Any;

use crate::Keystroke;

pub(crate) trait InputEvent {
  fn to_dene_input(self) -> DeneInput;
}
pub(crate) trait KeyboardEvent: InputEvent {}
pub(crate) trait MouseEvent: InputEvent {}

#[derive(Debug)]
pub(crate) enum DeneInput {
  KeyDown(KeyDownEvent),
  KeyUp(KeyUpEvent),
  MouseButtonDown(),
  MouseButtonUp(),
  MouseMove(),
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
      _ => None,
    }
  }
}

#[derive(Debug)]
pub(crate) struct KeyDownEvent {
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
pub(crate) struct KeyUpEvent {
  pub(crate) keystroke: Keystroke,
}
impl InputEvent for KeyUpEvent {
  fn to_dene_input(self) -> DeneInput {
    DeneInput::KeyUp(self)
  }
}
impl KeyboardEvent for KeyUpEvent {}
