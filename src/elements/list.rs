// SPDX-License-Identifier: Apache-2.0

use crate::{
  App, Component, Context, Entity, EventDispatcher, FocusHandle, Focusable,
  InteractiveElement, IntoElement, Keybind, Keystroke, ParentElement, Render,
  RenderOnce, StyleableElement, Window, div,
};

mod actions {
  use crate::actions;

  actions! {
    "list",
    [
      Escape,
      Enter,
      Prev,
      Next,
    ]
  }
}
use self::actions::*;

#[derive(Debug)]
#[derive(Clone, Copy)]
pub enum ListEvent {
  Submit(usize),
  Select(usize),
  Cancel,
}

#[derive(Debug)]
pub struct List<V>
where
  V: ListAdapter,
{
  state: Entity<ListState<V>>,
  style: taffy::Style,
}
impl<V> List<V>
where
  V: ListAdapter,
{
  pub fn new(state: &Entity<ListState<V>>) -> Self {
    Self {
      state: state.clone(),
      style: taffy::Style::DEFAULT,
    }
  }
}
impl<V> RenderOnce for List<V>
where
  V: ListAdapter,
{
  fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
    div().child(self.state.clone())
  }
}
impl<V> IntoElement for List<V>
where
  V: ListAdapter,
{
  type Element = Component<Self>;

  fn into_element(self) -> Self::Element {
    Component::new(self)
  }
}
impl<V> StyleableElement for List<V>
where
  V: ListAdapter,
{
  fn style(&mut self) -> &mut taffy::Style {
    &mut self.style
  }
}

#[derive(derive_more::Debug)]
pub struct ListState<V>
where
  V: ListAdapter,
{
  pub(crate) focus_handle: FocusHandle,
  selected_idx: Option<usize>,
  #[debug(skip)]
  adapter: V,
}
impl<V> ListState<V>
where
  V: ListAdapter,
{
  pub fn new(adapter: V, cx: &mut Context<Self>) -> Self {
    Self {
      focus_handle: cx.focus_handle(),
      selected_idx: None,
      adapter,
    }
  }
}
impl<V> Render for ListState<V>
where
  V: ListAdapter,
{
  fn render(
    &mut self,
    window: &mut Window,
    cx: &mut Context<Self>,
  ) -> impl IntoElement {
    div()
      .flex()
      .flex_col()
      .track_focus(&self.focus_handle)
      .children(
        (0..self.adapter.items_len())
          .flat_map(|idx| self.adapter.render_item(idx, window, cx)),
      )
  }
}
impl<V> Focusable for ListState<V>
where
  V: ListAdapter,
{
  fn focus_handle(&self, _: &App) -> FocusHandle {
    self.focus_handle.clone()
  }
}
impl<V> EventDispatcher<ListEvent> for ListState<V> where V: ListAdapter {}

pub trait ListAdapter: 'static + Sized {
  type Item: IntoElement;

  fn items_len(&self) -> usize;

  fn render_item(
    &mut self,
    idx: usize,
    window: &mut Window,
    cx: &mut Context<ListState<Self>>,
  ) -> Option<Self::Item> {
    None
  }
}

const KEY_CONTEXT: &str = "list";
pub(crate) fn init(cx: &mut App) {
  let key_context = Some(KEY_CONTEXT);
  cx.bind_keys([
    Keybind::new(Escape, [Keystroke::parse("esc")], key_context),
    Keybind::new(Enter, [Keystroke::parse("return")], key_context),
    Keybind::new(Prev, [Keystroke::parse("left")], key_context),
    Keybind::new(Next, [Keystroke::parse("right")], key_context),
    Keybind::new(Next, [Keystroke::parse("up")], key_context),
    Keybind::new(Prev, [Keystroke::parse("down")], key_context),
  ]);
}
