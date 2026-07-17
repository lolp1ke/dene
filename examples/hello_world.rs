// SPDX-License-Identifier: Apache-2.0

use std::rc::Rc;

use dene::{
  app::{App, AppContext, Application, Context},
  element::{
    InteractiveElement, IntoElement, ParentElement, Render, StyleableElement,
  },
  elements::{Delete, Input, InputState, div},
  entity::Entity,
  focus::{FocusHandle, Focusable},
  keybind::{Keybind, KeybindContextPredicate, Keystroke},
  window::Window,
};
use ropey::Rope;
use smallvec::smallvec;

fn main() {
  let app = Application::default();

  _ = app.run(|cx| {
    cx.open_window(Default::default(), |_window, cx| {
      cx.new_entity(HelloWorld::new)
    });

    cx.bind_key(Keybind {
      action: Box::new(Delete),
      keystrokes: smallvec![Keystroke::parse("delete").unwrap()],
      key_context: Some(Rc::new(KeybindContextPredicate::Ident(
        "input".into(),
      ))),
    });
  });

  #[cfg(debug_assertions)]
  dbg!(&app);
}

struct HelloWorld {
  focus_handle: FocusHandle,
  input: Entity<Input>,
}
impl HelloWorld {
  fn new(cx: &mut Context<Self>) -> Self {
    let input = cx.new_entity(|cx| Input {
      state: cx.new_entity(|cx| InputState {
        focus_handle: cx.focus_handle(),
        text: Rope::new(),
        placeholder: None,
        disabled: false,
      }),
    });

    Self {
      focus_handle: cx.focus_handle(),
      input,
    }
  }
}
impl Render for HelloWorld {
  fn render(
    &mut self,
    _window: &mut Window,
    _cx: &mut Context<Self>,
  ) -> impl IntoElement {
    div()
      .size_full()
      .track_focus(&self.focus_handle)
      .flex()
      .flex_col()
      .gap_y(10.)
      .items_center()
      .justify_center()
      .child("hello world")
      .child(
        div()
          .flex()
          .flex_row()
          .gap_x(5.)
          .border(1.)
          .child("one")
          .child("piece"),
      )
      .child(self.input.clone())
  }
}
impl Focusable for HelloWorld {
  fn focus_handle(&self, _: &App) -> FocusHandle {
    self.focus_handle.clone()
  }
}
