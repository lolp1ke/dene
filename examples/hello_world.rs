// SPDX-License-Identifier: Apache-2.0

use dene::{
  app::{App, AppContext, Application, Context},
  element::{
    InteractiveElement, IntoElement, ParentElement, Render, StyleableElement,
  },
  elements::div,
  focus::{FocusHandle, Focusable},
  window::Window,
};

fn main() {
  let app = Application::default();

  _ = app.run(|cx| {
    cx.open_window(Default::default(), |_window, cx| {
      cx.new_entity(|cx| HelloWorld {
        focus_handle: cx.focus_handle(),
      })
    });
  });

  #[cfg(debug_assertions)]
  dbg!(&app);
}

struct HelloWorld {
  focus_handle: FocusHandle,
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
      .on_key_down(|_, _, _| {
        println!("PRESS DETECTED");
      })
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
          .child("one")
          .child("piece"),
      )
  }
}
impl Focusable for HelloWorld {
  fn focus_handle(&self, _: &App) -> FocusHandle {
    self.focus_handle.clone()
  }
}
