// SPDX-License-Identifier: Apache-2.0

use dene::{
  app::{App, AppContext, Application, Context},
  color::rgb,
  element::{
    InteractiveElement, IntoElement, ParentElement, Render, StyleableElement,
  },
  elements::{Input, InputEvent, InputState, div},
  entity::Entity,
  focus::{FocusHandle, Focusable},
  window::Window,
};

fn main() {
  let app = Application::default();

  app.run(|cx| {
    cx.open_window(Default::default(), |_window, cx| {
      cx.new_entity(HelloWorld::new)
    });
  });
}

struct HelloWorld {
  focus_handle: FocusHandle,
  input: Entity<InputState>,
  input2: Entity<InputState>,

  search: String,
}
impl HelloWorld {
  fn new(cx: &mut Context<Self>) -> Self {
    let this = cx.entity();
    let input = cx.new_entity(InputState::new);
    let input2 = cx.new_entity(InputState::new);

    cx.on_event(&input, {
      let this = this.clone();

      move |_, event: &InputEvent, cx| {
        if let InputEvent::Submit(text) = event {
          this.update(cx, |this, _| {
            this.search = text.clone();
            tracing::info!("input submit event: {}", text);
          });
        };

        true
      }
    });

    Self {
      focus_handle: cx.focus_handle(),
      input,
      input2,
      search: String::new(),
    }
  }
}
impl Render for HelloWorld {
  fn render(
    &mut self,
    _: &mut Window,
    _: &mut Context<Self>,
  ) -> impl IntoElement {
    div()
      .size_full()
      .track_focus(&self.focus_handle)
      .tab_stop(true)
      .flex()
      .flex_col()
      .gap_y(3.)
      .items_center()
      .justify_center()
      .child("hello world")
      .child(
        div()
          .flex()
          .flex_row()
          .gap_x(5.)
          .border()
          .bg(rgb(0x0000ff))
          .child("one")
          .child("piece"),
      )
      .child(Input::new(&self.input))
      .child(Input::new(&self.input2).w_full())
  }
}
impl Focusable for HelloWorld {
  fn focus_handle(&self, _: &App) -> FocusHandle {
    self.focus_handle.clone()
  }
}
