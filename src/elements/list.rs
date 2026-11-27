// SPDX-License-Identifier: Apache-2.0

use std::rc::Rc;

use crate::{
  AnyElement, App, Component, Confirmable, Context, Disableable, Div,
  ElementExt, Entity, EventDispatcher, FocusHandle, Focusable, IndexPath,
  InteractiveElement, IntoElement, Keybind, Keystroke, ParentElement, Render,
  RenderOnce, ScrollAlignment, Selectable, StyleRefinement, StyleableElement,
  UniformListScrollHandle, Window, div, uniform_list,
};

use smallvec::SmallVec;

mod actions {
  use crate::actions;

  actions! {
    "list",
    [
      Prev,
      Next,
      Cancel,
      Submit,
    ]
  }
}
use actions::*;

pub struct List<A>
where
  A: ListAdapter,
{
  state: Entity<ListState<A>>,
  separator: Option<bool>,
}
impl<A> List<A>
where
  A: ListAdapter,
{
  #[must_use]
  pub fn new(state: &Entity<ListState<A>>) -> Self {
    Self {
      state: state.clone(),
      separator: None,
    }
  }

  #[must_use]
  pub fn separator(mut self, separator: impl Into<Option<bool>>) -> Self {
    self.separator = separator.into();
    self
  }
}
impl<A> RenderOnce for List<A>
where
  A: 'static + ListAdapter,
{
  fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
    if let Some(separator) = self.separator {
      self
        .state
        .update(cx, |state, _| state.separator = separator);
    }

    div()
      .size_full()
      .min_h(0.)
      .flex()
      .flex_col()
      .map(|this| {
        this
          .on_action(window.listener(&self.state, ListState::prev))
          .on_action(window.listener(&self.state, ListState::next))
          .on_action(window.listener(&self.state, ListState::cancel))
          .on_action(window.listener(&self.state, ListState::submit))
      })
      .child(self.state)
  }
}
impl<A> IntoElement for List<A>
where
  A: 'static + ListAdapter,
{
  type Element = Component<Self>;

  fn into_element(self) -> Self::Element {
    Component::new(self)
  }
}

#[derive(Debug)]
pub struct ListItem {
  base: Div,
  children: SmallVec<[AnyElement; 2]>,
  kind: ListItemKind,
  style: StyleRefinement,

  disabled: bool,
  confirmed: bool,
  selected: bool,
}
impl ListItem {
  #[must_use]
  pub fn new() -> Self {
    Self {
      base: div().flex().flex_col(),
      children: SmallVec::new(),
      kind: ListItemKind::Item,
      style: StyleRefinement::default(),

      disabled: false,
      confirmed: false,
      selected: false,
    }
  }

  #[must_use]
  pub fn kind(mut self, kind: ListItemKind) -> Self {
    self.kind = kind;
    self
  }
}
impl StyleableElement for ListItem {
  fn style(&mut self) -> &mut StyleRefinement {
    &mut self.style
  }
}
impl ParentElement for ListItem {
  fn extend<Iter>(&mut self, children: Iter)
  where
    Iter: IntoIterator,
    Iter::Item: IntoElement,
  {
    self
      .children
      .extend(children.into_iter().map(IntoElement::into_any_element));
  }
}
impl RenderOnce for ListItem {
  fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
    self
      .base
      .relative()
      .items_center()
      .justify_between()
      .w_full()
      .refine_style(&self.style)
      .when(self.kind == ListItemKind::Item, |this| {
        this.children(self.children)
      })
      .when(self.kind == ListItemKind::Separator, |this| {
        this.child(div().self_stretch().mx(2.).border_t())
      })
  }
}
impl IntoElement for ListItem {
  type Element = Component<Self>;
  fn into_element(self) -> Self::Element {
    Component::new(self)
  }
}
impl Selectable for ListItem {
  fn selected(mut self, selected: bool) -> Self {
    self.selected = selected;
    self
  }

  fn is_selected(&self) -> bool {
    self.selected
  }
}
impl Confirmable for ListItem {
  fn confirmed(mut self, confirmed: bool) -> Self {
    self.confirmed = confirmed;
    self
  }

  fn is_confirmed(&self) -> bool {
    self.confirmed
  }
}
impl Disableable for ListItem {
  fn disabled(mut self, disabled: bool) -> Self {
    self.disabled = disabled;
    self
  }

  fn is_disabled(&self) -> bool {
    self.disabled
  }
}

impl Default for ListItem {
  fn default() -> Self {
    Self::new()
  }
}

#[derive(Debug)]
#[derive(PartialEq)]
pub enum ListItemKind {
  Item,
  Separator,
}

pub struct ListState<A>
where
  A: ListAdapter,
{
  focus_handle: FocusHandle,
  adapter: A,
  rows: Rows,
  scroll_handle: UniformListScrollHandle,

  selected_idx: Option<IndexPath>,
  separator: bool,
  selectable: bool,
}
impl<A> ListState<A>
where
  A: 'static + ListAdapter,
{
  pub fn new(adapter: A, _: &mut Window, cx: &mut Context<Self>) -> Self {
    Self {
      focus_handle: cx.focus_handle(),
      adapter,
      rows: Rows::default(),
      scroll_handle: UniformListScrollHandle::new(),
      selected_idx: None,
      separator: false,
      selectable: true,
    }
  }

  pub fn prepare_rows(&mut self) {
    let sections_count = self.adapter.sections_count();
    let mut entities = Vec::new();
    let mut sections = Vec::with_capacity(sections_count);
    let mut items_count = 0;

    for section in 0..sections_count {
      let count = self.adapter.items_count(section);
      sections.push(count);
      items_count += count;
      if count == 0 {
        continue;
      }

      let has_header = self.adapter.has_header(section);
      if self.separator && !entities.is_empty() {
        let prev_was_footer =
          matches!(entities.last(), Some(RowEntry::Footer(..)));
        if prev_was_footer || has_header {
          entities.push(RowEntry::Separator);
        }
      }

      if has_header {
        entities.push(RowEntry::Header(section));
      }
      for row in 0..count {
        entities.push(RowEntry::Entry(IndexPath {
          section,
          row,
          col: 0,
        }));
      }
      if self.adapter.has_footer(section) {
        entities.push(RowEntry::Footer(section));
      }
    }
    self.rows = Rows {
      entities: Rc::new(entities),
      items_count,
      sections: Rc::new(sections),
    };
  }

  fn render_list_item(
    &mut self,
    idx_path: IndexPath,
    window: &mut Window,
    cx: &mut Context<Self>,
  ) -> impl IntoElement {
    div()
      .relative()
      .w_full()
      .overflow_hidden()
      .children(self.adapter.render_item(idx_path, window, cx))
  }

  pub(crate) fn render_items(
    &mut self,
    items_count: usize,
    entities_count: usize,
    window: &mut Window,
    cx: &mut Context<Self>,
  ) -> impl IntoElement {
    let rows = self.rows.clone();

    div()
      .size_full()
      .min_h(0.)
      .flex()
      .flex_col()
      .flex_grow(1.)
      .relative()
      .when(items_count == 0, |this| {
        this.child(self.adapter.render_empty(window, cx))
      })
      .when(items_count > 0, |this| {
        this.child(uniform_list(
          &cx.entity(),
          entities_count,
          &self.scroll_handle,
          move |state, range, window, cx| {
            range
              .map(|idx| {
                let Some(row_entry) = rows.get(idx) else {
                  return div();
                };

                div().children(match row_entry {
                  RowEntry::Entry(idx_path) => Some(
                    state
                      .render_list_item(idx_path, window, cx)
                      .into_any_element(),
                  ),
                  RowEntry::Header(section_idx) => state
                    .adapter
                    .render_header(section_idx, window, cx)
                    .map(IntoElement::into_any_element),
                  RowEntry::Footer(section_idx) => state
                    .adapter
                    .render_footer(section_idx, window, cx)
                    .map(IntoElement::into_any_element),
                  RowEntry::Separator => Some(
                    ListItem::new()
                      .kind(ListItemKind::Separator)
                      .into_any_element(),
                  ),
                })
              })
              .collect::<Vec<_>>()
          },
        ))
      })
  }
}
impl<A> ListState<A>
where
  A: ListAdapter,
{
  #[must_use]
  pub fn separator(mut self, separator: bool) -> Self {
    self.separator = separator;
    self
  }

  #[must_use]
  pub fn selectable(mut self, selectable: bool) -> Self {
    self.selectable = selectable;
    self
  }
}
impl<A> ListState<A>
where
  A: ListAdapter,
{
  fn select_item(
    &mut self,
    idx_path: IndexPath,
    window: &mut Window,
    cx: &mut Context<Self>,
  ) {
    if !self.selectable {
      return;
    }

    self.selected_idx = Some(idx_path);
    self.adapter.set_selected_idx(Some(idx_path), cx);
    if let Some(pos) = self.rows.position_of(idx_path) {
      self
        .scroll_handle
        .scroll_to_item(pos, ScrollAlignment::Nearest);
    }
    cx.emit(ListEvent::Select(idx_path));
    cx.notify();
  }

  #[allow(clippy::trivially_copy_pass_by_ref)]
  fn prev(&mut self, _: &Prev, window: &mut Window, cx: &mut Context<Self>) {
    if self.rows.len() == 0 {
      return;
    }

    if let Some(prev_idx) = self.rows.prev(self.selected_idx) {
      self.select_item(prev_idx, window, cx);
    }
  }
  #[allow(clippy::trivially_copy_pass_by_ref)]
  fn next(&mut self, _: &Next, window: &mut Window, cx: &mut Context<Self>) {
    if self.rows.len() == 0 {
      return;
    }

    if let Some(next_idx) = self.rows.next(self.selected_idx) {
      self.select_item(next_idx, window, cx);
    }
  }
  #[allow(clippy::trivially_copy_pass_by_ref)]
  fn cancel(
    &mut self,
    _: &Cancel,
    window: &mut Window,
    cx: &mut Context<Self>,
  ) {
    self.selected_idx = None;
    self.adapter.set_selected_idx(None, cx);
    self.adapter.cancel(window, cx);
    cx.emit(ListEvent::Clear);
    cx.notify();
  }
  #[allow(clippy::trivially_copy_pass_by_ref)]
  fn submit(
    &mut self,
    _: &Submit,
    window: &mut Window,
    cx: &mut Context<Self>,
  ) {
    if self.rows.len() == 0 {
      return;
    }

    let Some(idx) = self.selected_idx else {
      return;
    };

    self.adapter.set_selected_idx(self.selected_idx, cx);
    self.adapter.confirm(window, cx);
    cx.emit(ListEvent::Submit(idx));
    cx.notify();
  }
}

impl<A> Focusable for ListState<A>
where
  A: 'static + ListAdapter,
{
  fn focus_handle(&self, _: &App) -> FocusHandle {
    self.focus_handle.clone()
  }
}
impl<A> Render for ListState<A>
where
  A: 'static + ListAdapter,
{
  fn render(
    &mut self,
    window: &mut Window,
    cx: &mut Context<Self>,
  ) -> impl IntoElement {
    self.prepare_rows();
    let loading = self.adapter.loading(cx);

    let items_count = self.rows.items_count;
    let entities_count = self.rows.len();

    div()
      .key_context(KEY_CONTEXT)
      .tab_stop(true)
      .track_focus(&self.focus_handle)
      .size_full()
      .min_h(0.)
      .flex()
      .flex_col()
      .relative()
      .overflow_hidden()
      .when(!loading, |this| {
        this.child(self.render_items(items_count, entities_count, window, cx))
      })
      .when(loading, |this| {
        this.children(self.adapter.render_loading(window, cx))
      })
  }
}

#[allow(unused)]
pub trait ListAdapter {
  type Item: IntoElement;

  fn sections_count(&self) -> usize {
    1
  }
  fn items_count(&mut self, section: usize) -> usize;

  fn set_selected_idx(
    &mut self,
    idx_path: Option<IndexPath>,
    cx: &mut Context<ListState<Self>>,
  ) where
    Self: Sized;

  fn confirm(&mut self, window: &mut Window, cx: &mut Context<ListState<Self>>)
  where
    Self: Sized,
  {
  }
  fn cancel(&mut self, window: &mut Window, cx: &mut Context<ListState<Self>>)
  where
    Self: Sized,
  {
  }

  fn loading(&self, cx: &App) -> bool {
    false
  }
  fn render_loading(
    &mut self,
    window: &mut Window,
    cx: &mut Context<ListState<Self>>,
  ) -> Option<impl IntoElement>
  where
    Self: Sized,
  {
    None::<AnyElement>
  }

  fn render_item(
    &mut self,
    idx_path: IndexPath,
    window: &mut Window,
    cx: &mut Context<ListState<Self>>,
  ) -> Option<Self::Item>
  where
    Self: Sized,
  {
    None
  }
  fn render_header(
    &mut self,
    section_idx: usize,
    window: &mut Window,
    cx: &mut Context<ListState<Self>>,
  ) -> Option<impl IntoElement>
  where
    Self: Sized,
  {
    None::<AnyElement>
  }
  fn render_footer(
    &mut self,
    section_idx: usize,
    window: &mut Window,
    cx: &mut Context<ListState<Self>>,
  ) -> Option<impl IntoElement>
  where
    Self: Sized,
  {
    None::<AnyElement>
  }

  fn render_empty(
    &mut self,
    window: &mut Window,
    cx: &mut Context<ListState<Self>>,
  ) -> impl IntoElement
  where
    Self: Sized,
  {
    div().child("empty list")
  }

  fn has_header(&self, section: usize) -> bool {
    false
  }
  fn has_footer(&self, section: usize) -> bool {
    false
  }
}

#[derive(Debug)]
#[derive(Clone)]
#[derive(Default)]
struct Rows {
  entities: Rc<Vec<RowEntry>>,
  items_count: usize,
  sections: Rc<Vec<usize>>,
}
impl Rows {
  fn get(&self, idx: usize) -> Option<RowEntry> {
    self.entities.get(idx).copied()
  }
  fn len(&self) -> usize {
    self.entities.len()
  }

  fn position_of(&self, path: IndexPath) -> Option<usize> {
    self.entities.iter().position(
      |row| matches!(row, RowEntry::Entry(idx_path) if *idx_path == path),
    )
  }

  fn first_entry(&self) -> Option<IndexPath> {
    self.entities.iter().find_map(RowEntry::entry_idx)
  }
  fn last_entry(&self) -> Option<IndexPath> {
    self.entities.iter().rev().find_map(RowEntry::entry_idx)
  }

  fn prev(&self, path: Option<IndexPath>) -> Option<IndexPath> {
    path
      .and_then(|path| self.position_of(path))
      .and_then(|pos| {
        self.entities[..pos]
          .iter()
          .rev()
          .find_map(RowEntry::entry_idx)
      })
      .or_else(|| self.last_entry())
  }
  fn next(&self, path: Option<IndexPath>) -> Option<IndexPath> {
    path
      .and_then(|path| self.position_of(path))
      .and_then(|pos| {
        self.entities[pos + 1..]
          .iter()
          .find_map(RowEntry::entry_idx)
      })
      .or_else(|| self.first_entry())
  }
}

#[derive(Debug)]
#[derive(Clone, Copy)]
enum RowEntry {
  Entry(IndexPath),
  Header(usize),
  Footer(usize),
  Separator,
}
impl RowEntry {
  fn entry_idx(&self) -> Option<IndexPath> {
    match self {
      Self::Entry(idx) => Some(*idx),
      _ => None,
    }
  }
}

#[derive(Debug)]
pub enum ListEvent {
  Select(IndexPath),
  Clear,
  Submit(IndexPath),
}
impl<A> EventDispatcher<ListEvent> for ListState<A> where A: ListAdapter {}

const KEY_CONTEXT: &str = "list";
pub(crate) fn init(cx: &mut App) {
  let key_context = Some(KEY_CONTEXT);
  cx.bind_keys([
    Keybind::new(Prev, [Keystroke::parse("up")], key_context),
    Keybind::new(Next, [Keystroke::parse("down")], key_context),
    Keybind::new(Prev, [Keystroke::parse("left")], key_context),
    Keybind::new(Next, [Keystroke::parse("right")], key_context),
    Keybind::new(Cancel, [Keystroke::parse("esc")], key_context),
    Keybind::new(Submit, [Keystroke::parse("return")], key_context),
  ]);
}
