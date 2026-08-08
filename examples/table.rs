// SPDX-License-Identifier: Apache-2.0

use dene::{
  app::{AppContext, Application, Context},
  element::{ElementExt, IntoElement, ParentElement, Render, StyleableElement},
  elements::{TableAdapter, TableColumn, TableState, div},
  entity::Entity,
  style::TextAlign,
  window::Window,
};

fn main() {
  let app = Application::new();

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
    div()
      .flex()
      .flex_col()
      .gap_y(3.)
      .child("Top titles")
      .child(self.table.clone())
  }
}

#[derive(Clone)]
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
    let mut titles = vec![
      Title {
        name: "one piece".to_string(),
        score: 10.,
        episodes: 1172,
      };
      30
    ];
    titles.extend(vec![
      Title {
        name: "one piece S2".to_string(),
        score: 10.,
        episodes: 1172,
      };
      30
    ]);

    Self {
      columns: vec![
        TableColumn::new("name", "name"),
        TableColumn::new("score", "score"),
        TableColumn::new("episode_count", "episodes"),
      ],
      titles,
    }
  }
}
impl TableAdapter for TitlesAdapter {
  fn columns_count(&self) -> usize {
    self.columns.len()
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

  fn render_td(
    &mut self,
    row_idx: usize,
    col_idx: usize,
    window: &mut Window,
    cx: &mut Context<TableState<Self>>,
  ) -> impl IntoElement {
    let title = self.titles.get(row_idx).unwrap();
    let Some(col) = self.columns.get(col_idx) else {
      return div().child("---");
    };

    match &*col.name {
      "name" => div()
        .when(matches!(col.align, TextAlign::Center), |this| {
          this.justify_center()
        })
        .when(matches!(col.align, TextAlign::Right), |this| {
          this.justify_end()
        })
        .child(&title.name),

      "score" => div()
        .when(matches!(col.align, TextAlign::Center), |this| {
          this.justify_center()
        })
        .when(matches!(col.align, TextAlign::Right), |this| {
          this.justify_end()
        })
        .child(title.score),

      "episode_count" => div()
        .when(matches!(col.align, TextAlign::Center), |this| {
          this.justify_center()
        })
        .when(matches!(col.align, TextAlign::Right), |this| {
          this.justify_end()
        })
        .child(title.episodes),

      _ => div().child("---"),
    }
  }
}
