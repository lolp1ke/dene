// SPDX-License-Identifier: Apache-2.0

use dene::{
  app::{AppContext, Application, Context},
  color::rgb,
  element::{
    InteractiveElement, IntoElement, ParentElement, Render, StyleableElement,
  },
  elements::{Input, InputState, UniformList, UniformListScrollHandle, div},
  entity::Entity,
  focus::FocusHandle,
  window::Window,
};

fn main() {
  Application::new().run(|cx| {
    cx.open_window(Default::default(), |_, cx| cx.new_entity(Backgrounds::new));
  });
}

struct Backgrounds {
  panel_focus: FocusHandle,
  input: Entity<InputState>,
  scroll: UniformListScrollHandle,
}

impl Backgrounds {
  fn new(cx: &mut Context<Self>) -> Self {
    Self {
      panel_focus: cx.focus_handle(),
      input: cx.new_entity(InputState::new),
      scroll: UniformListScrollHandle::new(),
    }
  }
}

impl Render for Backgrounds {
  fn render(
    &mut self,
    _: &mut Window,
    _: &mut Context<Self>,
  ) -> impl IntoElement {
    div()
      .size_full()
      .flex()
      .flex_col()
      .p(1.)
      .gap_y(1.)
      .bg(rgb(0x111111))
      .child(
        div()
          .flex()
          .flex_col()
          .flex_shrink(0.)
          .border()
          .px(1.)
          .bg(rgb(0x222222))
          .track_focus(&self.panel_focus)
          .tab_index(0)
          .focused(|style| style.bg(rgb(0x666666)))
          .child("focused panel")
          .child(
            div()
              .flex()
              .gap_x(2.)
              .child(div().px(1.).bg(rgb(0xff0000)).child("red"))
              .child(div().px(1.).bg(rgb(0x00ff00)).child("green"))
              .child(div().px(1.).bg(rgb(0x0000ff)).child("blue")),
          ),
      )
      .child(Input::new(&self.input).tab_index(1).bg(rgb(0x222222)))
      .child(
        UniformList::new(100, &self.scroll, |range, _, _| {
          range.map(|index| {
            let bg = if index % 2 == 0 { 0xffffff } else { 0xff00ff };
            div()
              .w_full()
              .px(1.)
              .bg(rgb(bg))
              .child(format!("row {index:03} — scroll here"))
          })
        })
        .border()
        .min_h(5.)
        .max_h(5.)
        .bg(rgb(0x222222)),
      )
  }
}
