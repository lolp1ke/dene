// SPDX-License-Identifier: Apache-2.0

use std::sync::Arc;

use crate::{
  AnyElement, App, AppContext, Component, Context, Div, ElementExt, Entity,
  EventDispatcher, FocusHandle, Focusable, InteractiveElement, IntoElement,
  Keybind, Keystroke, ParentElement, Render, RenderOnce, ScrollHandle,
  StyleableElement, TextAlign, Window, div,
};

mod actions {
  use crate::actions;

  actions! {
    "table",
    [
      Left,
      Right,
      Up,
      Down,
      Escape,
      Enter,
    ]
  }
}
use actions::*;

const MIN_CELL_WIDTH: f32 = 8.0;

#[derive(Debug)]
#[derive(Clone, Copy)]
pub enum TableEvent {
  SelectedColumn(usize),
  SelectedRow(usize),
  SelectedCell(usize, usize),
  ClearSelected,
}

#[derive(Debug)]
pub struct Table<A>
where
  A: TableAdapter,
{
  state: Entity<TableState<A>>,
}
impl<A> Table<A>
where
  A: TableAdapter,
{
  pub fn new(state: &Entity<TableState<A>>) -> Self {
    Self {
      state: state.clone(),
    }
  }
}
impl<A> RenderOnce for Table<A>
where
  A: TableAdapter,
{
  fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
    let state = self.state.read(cx);

    div()
      .size_full()
      .key_context(KEY_CONTEXT)
      .track_focus(&state.focus_handle)
      .child(self.state)
  }
}
impl<A> IntoElement for Table<A>
where
  A: TableAdapter,
{
  type Element = Component<Self>;

  fn into_element(self) -> Self::Element {
    Component::new(self)
  }
}

#[derive(Debug)]
pub struct TableDummy {
  style: taffy::Style,
  children: Vec<AnyElement>,
}
impl RenderOnce for TableDummy {
  fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
    div().children(self.children)
  }
}
impl IntoElement for TableDummy {
  type Element = Component<Self>;

  fn into_element(self) -> Self::Element {
    Component::new(self)
  }
}
impl StyleableElement for TableDummy {
  fn style(&mut self) -> &mut taffy::Style {
    &mut self.style
  }
}
impl ParentElement for TableDummy {
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

#[derive(Debug)]
pub struct TableHeader {
  style: taffy::Style,
  children: Vec<AnyElement>,
}
impl RenderOnce for TableHeader {
  fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
    div()
  }
}
impl IntoElement for TableHeader {
  type Element = Component<Self>;

  fn into_element(self) -> Self::Element {
    Component::new(self)
  }
}
impl StyleableElement for TableHeader {
  fn style(&mut self) -> &mut taffy::Style {
    &mut self.style
  }
}
impl ParentElement for TableHeader {
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

#[derive(Debug)]
pub struct TableBody {
  style: taffy::Style,
  children: Vec<AnyElement>,
}
impl RenderOnce for TableBody {
  fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
    div()
  }
}
impl IntoElement for TableBody {
  type Element = Component<Self>;

  fn into_element(self) -> Self::Element {
    Component::new(self)
  }
}
impl StyleableElement for TableBody {
  fn style(&mut self) -> &mut taffy::Style {
    &mut self.style
  }
}
impl ParentElement for TableBody {
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

#[derive(Debug)]
pub struct TableFooter {
  style: taffy::Style,
  children: Vec<AnyElement>,
}
impl RenderOnce for TableFooter {
  fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
    div()
  }
}
impl IntoElement for TableFooter {
  type Element = Component<Self>;

  fn into_element(self) -> Self::Element {
    Component::new(self)
  }
}
impl StyleableElement for TableFooter {
  fn style(&mut self) -> &mut taffy::Style {
    &mut self.style
  }
}
impl ParentElement for TableFooter {
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

#[derive(Debug)]
pub struct TableRow {
  style: taffy::Style,
  children: Vec<AnyElement>,
}
impl RenderOnce for TableRow {
  fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
    div()
      .map(|mut this| {
        *this.style() = self.style;
        this
      })
      .flex()
      .flex_row()
      .children(self.children)
  }
}
impl IntoElement for TableRow {
  type Element = Component<Self>;

  fn into_element(self) -> Self::Element {
    Component::new(self)
  }
}
impl StyleableElement for TableRow {
  fn style(&mut self) -> &mut taffy::Style {
    &mut self.style
  }
}
impl ParentElement for TableRow {
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

#[derive(Debug)]
pub struct TableHead {
  style: taffy::Style,
  children: Vec<AnyElement>,
  col_span: usize,
  align: TextAlign,
}
impl RenderOnce for TableHead {
  fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
    div().children(self.children)
  }
}
impl IntoElement for TableHead {
  type Element = Component<Self>;

  fn into_element(self) -> Self::Element {
    Component::new(self)
  }
}
impl StyleableElement for TableHead {
  fn style(&mut self) -> &mut taffy::Style {
    &mut self.style
  }
}
impl ParentElement for TableHead {
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

#[derive(Debug)]
pub struct TableCell {
  style: taffy::Style,
  children: Vec<AnyElement>,
  align: TextAlign,
}
impl TableCell {
  pub fn new() -> Self {
    Self {
      style: taffy::Style::DEFAULT,
      children: Vec::new(),
      align: TextAlign::Left,
    }
  }

  pub fn text_left(mut self) -> Self {
    self.align = TextAlign::Left;
    self
  }
  pub fn text_center(mut self) -> Self {
    self.align = TextAlign::Center;
    self
  }
  pub fn text_right(mut self) -> Self {
    self.align = TextAlign::Right;
    self
  }
}
impl RenderOnce for TableCell {
  fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
    div()
      .map(|mut this| {
        *this.style() = self.style;
        this
      })
      .flex()
      .items_center()
      .when(matches!(self.align, TextAlign::Center), |this| {
        this.justify_center()
      })
      .when(matches!(self.align, TextAlign::Right), |this| {
        this.justify_end()
      })
      .children(self.children)
  }
}
impl IntoElement for TableCell {
  type Element = Component<Self>;

  fn into_element(self) -> Self::Element {
    Component::new(self)
  }
}
impl StyleableElement for TableCell {
  fn style(&mut self) -> &mut taffy::Style {
    &mut self.style
  }
}
impl ParentElement for TableCell {
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
impl Default for TableCell {
  fn default() -> Self {
    Self::new()
  }
}

#[derive(Debug)]
#[derive(Clone, Copy)]
pub enum TableColumnSort {
  Default,
  Descending,
  Ascending,
}

#[derive(Debug)]
#[derive(Clone)]
#[derive(Default)]
pub struct TableColumn {
  pub key: Arc<str>,
  pub name: Arc<str>,
  pub align: TextAlign,
  pub sort: Option<TableColumnSort>,

  pub width: f64,
  pub min_width: f64,
  pub max_width: f64,
}
impl TableColumn {
  pub fn new<S1, S2>(key: S1, name: S2) -> Self
  where
    S1: Into<Arc<str>>,
    S2: Into<Arc<str>>,
  {
    Self {
      key: key.into(),
      name: name.into(),
      ..Default::default()
    }
  }
}

#[derive(derive_more::Debug)]
pub struct TableState<A>
where
  A: TableAdapter,
{
  focus_handle: FocusHandle,
  scroll_handle: ScrollHandle,
  #[debug(skip)]
  adapter: A,
}
impl<A> TableState<A>
where
  A: TableAdapter,
{
  pub fn new(adapter: A, cx: &mut Context<Self>) -> Self {
    Self {
      focus_handle: cx.focus_handle(),
      scroll_handle: ScrollHandle::new([
        taffy::Overflow::Scroll,
        taffy::Overflow::Scroll,
      ]),
      adapter,
    }
  }

  pub fn adapter(&self) -> &A {
    &self.adapter
  }
  pub fn adater_mut(&mut self) -> &mut A {
    &mut self.adapter
  }
}
impl<A> EventDispatcher<TableEvent> for TableState<A> where A: TableAdapter {}
impl<A> Focusable for TableState<A>
where
  A: TableAdapter,
{
  fn focus_handle(&self, _: &App) -> FocusHandle {
    self.focus_handle.clone()
  }
}
impl<A> Render for TableState<A>
where
  A: TableAdapter,
{
  fn render(
    &mut self,
    window: &mut Window,
    cx: &mut Context<Self>,
  ) -> impl IntoElement {
    let cols = self.adapter.columns_count();
    let rows = self.adapter.rows_count();
    let col_meta: Vec<(f64, f64, f64, TextAlign)> = (0..cols)
      .map(|i| {
        let col = self.adapter.column(i);
        (col.width, col.min_width, col.max_width, col.align)
      })
      .collect();

    let cell = |(w, min_w, max_w, align): (f64, f64, f64, TextAlign),
                content: AnyElement| {
      div()
        .flex()
        .items_center()
        .overflow_hidden()
        .min_w(MIN_CELL_WIDTH)
        .when(w == 0.0, |this| {
          this.flex_basis(0.).flex_grow(1.).flex_shrink(1.)
        })
        .when(w > 0.0, |mut this| {
          this.style().size.width = taffy::Dimension::length(w as f32);
          this
        })
        .when(min_w > 0.0, |mut this| {
          this.style().min_size.width = taffy::Dimension::length(min_w as f32);
          this
        })
        .when(max_w > 0.0, |mut this| {
          this.style().max_size.width = taffy::Dimension::length(max_w as f32);
          this
        })
        .when(matches!(align, TextAlign::Center), |this| {
          this.justify_center()
        })
        .when(matches!(align, TextAlign::Right), |this| this.justify_end())
        .child(content)
    };

    let handle = self.scroll_handle.clone();
    let body = div()
      .flex()
      .flex_col()
      .w_full()
      .flex_grow(1.)
      .flex_basis(0.)
      .min_h(0.)
      .overflow_hidden()
      .track_scroll(&self.scroll_handle)
      .on_scroll_wheel(move |event, _, _| {
        let inner = handle.0.borrow();
        let max_y = inner
          .content_size
          .height
          .saturating_sub(inner.bounds.height);
        let mut offset = inner.offset.borrow_mut();
        offset.y = (offset.y as i32 + event.scroll_delta * 3)
          .clamp(0, max_y as i32) as u16;
      })
      .children((0..rows).map(|r| {
        div()
          .flex()
          .flex_row()
          .w_full()
          .children((0..cols).map(|c| {
            cell(
              col_meta[c],
              self.adapter.render_td(r, c, window, cx).into_any_element(),
            )
            .into_any_element()
          }))
          .into_any_element()
      }));

    div()
      .flex()
      .flex_col()
      .size_full()
      .overflow_hidden()
      .child(
        div().w_full().flex_shrink(0.).child(
          div()
            .flex()
            .flex_row()
            .w_full()
            .children((0..cols).map(|c| {
              cell(
                col_meta[c],
                self.adapter.render_th(c, window, cx).into_any_element(),
              )
              .into_any_element()
            })),
        ),
      )
      .child(body)
  }
}

pub trait TableAdapter: 'static + Sized {
  fn columns_count(&self) -> usize;
  fn rows_count(&self) -> usize;
  fn column(&self, idx: usize) -> TableColumn;

  fn render_th(
    &mut self,
    idx: usize,
    window: &mut Window,
    cx: &mut Context<TableState<Self>>,
  ) -> impl IntoElement {
    div().size_full().child(self.column(idx).name)
  }
  fn render_td(
    &mut self,
    row_idx: usize,
    col_idx: usize,
    window: &mut Window,
    cx: &mut Context<TableState<Self>>,
  ) -> impl IntoElement;
}

const KEY_CONTEXT: &str = "table";

pub(crate) fn init(cx: &mut App) {
  let key_context = Some(KEY_CONTEXT);
  cx.bind_keys([
    Keybind::new(Left, [Keystroke::parse("left")], key_context),
    Keybind::new(Left, [Keystroke::parse("h")], key_context),
    Keybind::new(Right, [Keystroke::parse("right")], key_context),
    Keybind::new(Right, [Keystroke::parse("l")], key_context),
    Keybind::new(Up, [Keystroke::parse("up")], key_context),
    Keybind::new(Up, [Keystroke::parse("k")], key_context),
    Keybind::new(Down, [Keystroke::parse("down")], key_context),
    Keybind::new(Down, [Keystroke::parse("j")], key_context),
    Keybind::new(Escape, [Keystroke::parse("esc")], key_context),
    Keybind::new(Enter, [Keystroke::parse("return")], key_context),
  ]);
}
