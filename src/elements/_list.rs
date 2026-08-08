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
pub struct List<A>
where
  A: ListAdapter,
{
  state: Entity<ListState<A>>,
  style: taffy::Style,
  tab_index: isize,
}
impl<A> List<A>
where
  A: ListAdapter,
{
  pub fn new(state: &Entity<ListState<A>>) -> Self {
    Self {
      state: state.clone(),
      style: taffy::Style::DEFAULT,
      tab_index: 0,
    }
  }
}
impl<A> RenderOnce for List<A>
where
  A: ListAdapter,
{
  fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
    let state = self.state.read(cx);

    div()
      .key_context(KEY_CONTEXT)
      .track_focus(&state.focus_handle)
      .tab_index(self.tab_index)
      .on_action(window.listener(&self.state, ListState::submit))
      .on_action(window.listener(&self.state, ListState::next))
      .on_action(window.listener(&self.state, ListState::prev))
      .on_action(window.listener(&self.state, ListState::cancel))
      .child(self.state.clone())
  }
}
impl<A> IntoElement for List<A>
where
  A: ListAdapter,
{
  type Element = Component<Self>;

  fn into_element(self) -> Self::Element {
    Component::new(self)
  }
}
impl<A> StyleableElement for List<A>
where
  A: ListAdapter,
{
  fn style(&mut self) -> &mut taffy::Style {
    &mut self.style
  }
}

#[derive(derive_more::Debug)]
pub struct ListState<A>
where
  A: ListAdapter,
{
  pub(crate) focus_handle: FocusHandle,
  selected_idx: Option<usize>,
  #[debug(skip)]
  adapter: A,
}
impl<A> ListState<A>
where
  A: ListAdapter,
{
  pub fn new(adapter: A, cx: &mut Context<Self>) -> Self {
    Self {
      focus_handle: cx.focus_handle(),
      selected_idx: None,
      adapter,
    }
  }

  pub fn adapter(&self) -> &A {
    &self.adapter
  }
  pub fn adapter_mut(&mut self) -> &mut A {
    &mut self.adapter
  }

  fn submit(&mut self, _: &Enter, _: &mut Window, cx: &mut Context<Self>) {
    if let Some(selected_idx) = self.selected_idx {
      if !cx.prevent_default {
        self.selected_idx = None;
      };
      cx.emit(ListEvent::Submit(selected_idx));
    };
  }
  fn prev(&mut self, _: &Prev, _: &mut Window, cx: &mut Context<Self>) {
    let len = self.adapter().items_len();
    if len == 0 {
      self.selected_idx = None;
      return;
    };

    let idx = match self.selected_idx {
      Some(idx) => (idx + 1) % len,
      None => 0,
    };
    self.select(idx, cx);
  }
  fn next(&mut self, _: &Next, _: &mut Window, cx: &mut Context<Self>) {
    let len = self.adapter().items_len();
    if len == 0 {
      self.selected_idx = None;
      return;
    };

    let idx = match self.selected_idx {
      Some(idx) => {
        if idx == 0 {
          len - 1
        } else {
          idx - 1
        }
      }
      None => len - 1,
    };
    self.select(idx, cx);
  }
  fn cancel(&mut self, _: &Escape, _: &mut Window, cx: &mut Context<Self>) {
    self.selected_idx = None;
    cx.emit(ListEvent::Cancel);
  }

  fn select(&mut self, idx: usize, cx: &mut Context<Self>) {
    self.selected_idx = Some(idx);
    cx.emit(ListEvent::Select(idx));
  }
}
impl<A> Render for ListState<A>
where
  A: ListAdapter,
{
  fn render(
    &mut self,
    window: &mut Window,
    cx: &mut Context<Self>,
  ) -> impl IntoElement {
    div().flex().flex_col().children(
      (0..self.adapter.items_len())
        .flat_map(|idx| self.adapter.render_item(idx, window, cx)),
    )
  }
}
impl<A> Focusable for ListState<A>
where
  A: ListAdapter,
{
  fn focus_handle(&self, _: &App) -> FocusHandle {
    self.focus_handle.clone()
  }
}
impl<A> EventDispatcher<ListEvent> for ListState<A> where A: ListAdapter {}

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
