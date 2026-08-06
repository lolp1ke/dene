// SPDX-License-Identifier: Apache-2.0

use dene::{
  app::{AppContext, Application, Context},
  element::{IntoElement, ParentElement, Render, StyleableElement},
  elements::{TableAdapter, TableColumn, TableState, div},
  entity::Entity,
};

fn main() {
  let mut app = Application::new();

  app.run(move |cx| {
    cx.open_window(Default::default(), move |window, cx| {
      cx.new_entity(TopTitles::new)
    })
  });
}

struct TopTitles {
  table: Entity<TableState<TitlesAdapter>>,
}
impl TopTitles {
  fn new(cx: &mut Context<Self>) -> Self {
    let table =
      cx.new_entity(move |cx| TableState::new(TitlesAdapter::new(), cx));
    Self { table }
  }
}
impl Render for TopTitles {
  fn render(
    &mut self,
    window: &mut dene::window::Window,
    cx: &mut Context<Self>,
  ) -> impl IntoElement {
    div().flex().flex_col().gap_y(3.).child("Top titles")
  }
}

struct Title {
  name: String,
  score: f32,
  episodes: u32,
}
struct TitlesAdapter {
  columns: Vec<TableColumn>,
  titles: Vec<Title>,
}
impl TitlesAdapter {
  fn new() -> Self {
    Self {
      columns: Vec::new(),
      titles: Vec::new(),
    }
  }
}
impl TableAdapter for TitlesAdapter {
  fn columns_count(&self) -> usize {
    4
  }
  fn rows_count(&self) -> usize {
    self.titles.len()
  }
  fn column(&self, idx: usize) -> dene::elements::TableColumn {
    if let Some(col) = self.columns.get(idx) {
      col.clone()
    } else {
      TableColumn::new("non_existing", "non_existing")
    }
  }
}
