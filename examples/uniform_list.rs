// SPDX-License-Identifier: Apache-2.0

use dene::{
  app::{AppContext, Application, Context},
  element::{IntoElement, ParentElement, Render, StyleableElement},
  elements::{
    Axis, ScrollAlignment, UniformList, UniformListScrollHandle, div,
  },
  window::Window,
};

fn main() {
  let app = Application::new();

  app.run(|cx| {
    cx.open_window(Default::default(), |_, cx| {
      cx.new_entity(|_| {
        let vertical = UniformListScrollHandle::new();
        vertical.scroll_to_item(50000, ScrollAlignment::Start);
        Lists {
          vertical,
          horizontal: UniformListScrollHandle::new(),
        }
      })
    });
  });
}

struct Lists {
  vertical: UniformListScrollHandle,
  horizontal: UniformListScrollHandle,
}

impl Render for Lists {
  fn render(
    &mut self,
    _: &mut Window,
    _: &mut Context<Self>,
  ) -> impl IntoElement {
    let horizontal =
      UniformList::new(100000, &self.horizontal, |range, _, _| {
        range.map(|index| div().px(1.).child(format!("{index:06}")))
      })
      .axis(Axis::Horizontal)
      .border(1.)
      .min_h(3.)
      .max_h(3.);

    let vertical = UniformList::new(1000000, &self.vertical, |range, _, _| {
      range.map(|index| {
        div()
          .flex()
          .flex_col()
          .child(format!("Item {index:06}"))
          .child("  A measured two-line row")
      })
    })
    .border(1.);

    div().size_full().flex().flex_col()
      .child("UniformList: wheel over either list to scroll. Ctrl-; then Ctrl-Q quits.")
      .child(vertical)
      .child(horizontal)
  }
}
