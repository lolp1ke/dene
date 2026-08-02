// SPDX-License-Identifier: Apache-2.0

use dene::{
  app::{App, AppContext, Application, Context},
  element::{
    Component, InteractiveElement, IntoElement, ParentElement, Render,
    RenderOnce, StyleableElement,
  },
  elements::{
    Input, InputEvent, InputState, List, ListAdapter, ListEvent, ListState, div,
  },
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
  input: Entity<InputState>,
  list: Entity<ListState<HelloWorldListAdapter>>,

  search: String,
}
impl HelloWorld {
  fn new(cx: &mut Context<Self>) -> Self {
    let this = cx.entity();
    let input = cx.new_entity(InputState::new);
    let list = cx.new_entity(|cx| {
      ListState::new(
        HelloWorldListAdapter {
          items: vec![
            HelloWorldListItem {
              title: "one piece season 1".into(),
              description: "peak".into(),
            },
            HelloWorldListItem {
              title: "one piece season 2".into(),
              description: "also peak".into(),
            },
            HelloWorldListItem {
              title: "minecraft".into(),
              description: "wonderful game".into(),
            },
          ],
        },
        cx,
      )
    });

    cx.on_event(&input, {
      let this = this.clone();

      move |_, event: &InputEvent, cx| {
        if let InputEvent::Submit(text) = event {
          this.update(cx, |this, _| {
            this.search = text.clone();
          });
        };

        true
      }
    });
    cx.on_event(&list, move |_, event: &ListEvent, _| {
      if let ListEvent::Select(idx) = event {
        tracing::info!("current item idx: {}", idx);
      };

      true
    });
    cx.on_event(&list, move |_, event: &ListEvent, _| {
      if let ListEvent::Submit(idx) = event {
        tracing::info!("item: {} is being processed", idx);
      };

      true
    });

    Self {
      focus_handle: cx.focus_handle(),
      input,
      list,
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
      .child(Input::new(&self.input))
      .child(List::new(&self.list))
  }
}
impl Focusable for HelloWorld {
  fn focus_handle(&self, _: &App) -> FocusHandle {
    self.focus_handle.clone()
  }
}

struct HelloWorldListAdapter {
  items: Vec<HelloWorldListItem>,
}

#[derive(Clone)]
struct HelloWorldListItem {
  title: String,
  description: String,
}
impl RenderOnce for HelloWorldListItem {
  fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
    div()
      .grid()
      .grid_cols(2)
      .gap_x(3.)
      .child(&self.title)
      .child(&self.description)
  }
}
impl IntoElement for HelloWorldListItem {
  type Element = Component<Self>;

  fn into_element(self) -> Self::Element {
    Component::new(self)
  }
}
impl ListAdapter for HelloWorldListAdapter {
  type Item = HelloWorldListItem;

  fn items_len(&self) -> usize {
    self.items.len()
  }

  fn render_item(
    &mut self,
    idx: usize,
    _: &mut Window,
    _: &mut Context<ListState<Self>>,
  ) -> Option<Self::Item> {
    self.items.get(idx).cloned()
  }
}
