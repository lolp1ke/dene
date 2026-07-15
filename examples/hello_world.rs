// SPDX-License-Identifier: Apache-2.0

use dene::{
  app::{AppContext, Application, Context},
  element::{IntoElement, Render},
  elements::{Empty, div},
  window::Window,
};

fn main() {
  let app = Application::new();

  _ = app.run(|cx| {
    cx.open_window(Default::default(), |_window, cx| {
      cx.new_entity(|_| HelloWorld {})
    })
  })
}

struct HelloWorld {}
impl Render for HelloWorld {
  fn render(
    &mut self,
    _window: &mut Window,
    _cx: &mut Context<Self>,
  ) -> impl IntoElement {
    div()
  }
}
