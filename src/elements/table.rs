// SPDX-License-Identifier: Apache-2.0

use std::sync::Arc;

use crate::{
  AnyElement, App, Component, Context, ElementExt, Entity, FocusHandle,
  InteractiveElement, IntoElement, ParentElement, Render, RenderOnce,
  StyleableElement, TextAlign, Window, div,
};

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
  key: Arc<str>,
  name: Arc<str>,
  align: TextAlign,
  sort: Option<TableColumnSort>,

  width: u16,
  min_width: u16,
  max_width: u16,
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
      adapter,
    }
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
    div()
  }
}

pub trait TableAdapter: 'static + Sized {
  fn columns_count(&self) -> usize;
  fn rows_count(&self) -> usize;
  fn column(&self, idx: usize) -> TableColumn;
}

const KEY_CONTEXT: &str = "table";

pub(crate) fn init(cx: &mut App) {}
