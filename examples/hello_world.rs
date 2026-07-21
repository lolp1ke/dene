// SPDX-License-Identifier: Apache-2.0

use dene::{
  app::{App, AppContext, Application, Context},
  element::{
    InteractiveElement, IntoElement, ParentElement, Render, StyleableElement,
  },
  elements::{Input, div, input},
  entity::Entity,
  focus::{FocusHandle, Focusable},
  window::Window,
};

fn main() {
  let mut app = Application::default();

  _ = app.run(|cx| {
    cx.open_window(Default::default(), |_window, cx| {
      cx.new_entity(HelloWorld::new)
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
    let input = input(cx);

    Self {
      focus_handle: cx.focus_handle(),
      input,
    }
  }
}
impl Render for HelloWorld {
  fn render(
    &mut self,
    window: &mut Window,
    cx: &mut Context<Self>,
  ) -> impl IntoElement {
    let input = self.input.clone();
    window.handle_input(&input.focus_handle(cx), input);

    div()
      .size_full()
      .track_focus(&self.focus_handle)
      .tab_stop(true)
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
