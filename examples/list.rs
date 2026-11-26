// SPDX-License-Identifier: Apache-2.0

use std::{rc::Rc, sync::Arc};

use dene::{
  app::{App, AppContext, Application, Context},
  color::rgb,
  element::{
    Component, IntoElement, ParentElement, Render, RenderOnce, StyleableElement,
  },
  elements::{IndexPath, List, ListAdapter, ListItem, ListState, div},
  entity::Entity,
  window::{Window, WindowOptions},
};

fn main() {
  let app = Application::new();

  app.run(move |cx| {
    cx.open_window(WindowOptions::default(), move |window, cx| {
      cx.new_entity(|cx| ListExample::new(window, cx))
    })
  });
}

struct ListExample {
  list_state: Entity<ListState<TitlesListAdapter>>,
}
impl ListExample {
  fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
    let list_state = cx.new_entity(|cx| {
      ListState::new(
        TitlesListAdapter {
          sections: vec![
            Section {
              category: Category::Series,
              titles: vec![
                Title::new("breaking bad", "vince gilligan", 9.6),
                Title::new("andor", "tony gilroy", 9.6),
                Title::new("severance", "dan erickson", 9.6),
                Title::new("the wire", "david simon", 9.6),
              ],
            },
            Section {
              category: Category::Film,
              titles: vec![
                Title::new("spirited away", "hayao miyazaki", 9.6),
                Title::new("parasite", "bong joon-ho", 9.6),
                Title::new("dune: part two", "denis villeneuve", 9.6),
                Title::new("blade runner 2049", "denis villeneuve", 9.6),
              ],
            },
            Section {
              category: Category::Manga,
              titles: vec![
                Title::new("one piece", "eiichiro oda", 10.),
                Title::new("berserk", "kentaro miura", 9.6),
                Title::new("vagabond", "Takehiko Inoue", 9.6),
                Title::new("monster", "naoki urasawa", 9.6),
                Title::new("vinland saga", "makoto yukimura", 9.6),
              ],
            },
          ],
        },
        window,
        cx,
      )
    });
    Self { list_state }
  }
}
impl Render for ListExample {
  fn render(
    &mut self,
    _: &mut Window,
    _: &mut Context<Self>,
  ) -> impl IntoElement {
    div().size_full().flex().flex_col().child(
      div()
        .flex_grow(1.)
        .min_h(0.)
        .w_full()
        .border()
        .child(List::new(&self.list_state)),
    )
  }
}

#[derive(Debug)]
#[derive(Clone, Copy)]
enum Category {
  Series,
  Film,
  Manga,
}

struct Title {
  name: Arc<str>,
  author: Arc<str>,
  rating: f32,
}
impl Title {
  fn new(name: &str, author: &str, rating: f32) -> Rc<Self> {
    Rc::new(Self {
      name: name.into(),
      author: author.into(),
      rating,
    })
  }
}

struct Section {
  category: Category,
  titles: Vec<Rc<Title>>,
}
impl Section {
  fn avg_rating(&self) -> f64 {
    if self.titles.is_empty() {
      return 0.0;
    }
    let sum = f64::from(self.titles.iter().map(|t| t.rating).sum::<f32>());
    sum / self.titles.len() as f64
  }
}

struct TitleListItem {
  base: ListItem,
  title: Rc<Title>,
}
impl TitleListItem {
  fn new(title: Rc<Title>) -> Self {
    Self {
      base: ListItem::new(),
      title,
    }
  }
}
impl RenderOnce for TitleListItem {
  fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
    self.base.w_full().overflow_x_hidden().child(
      div()
        .w_full()
        .flex()
        .flex_row()
        .justify_between()
        .px(1.)
        .child(
          div()
            .flex()
            .flex_row()
            .gap_x(2.)
            .child(self.title.name.clone())
            .child(format!("by {}", self.title.author)),
        )
        .child(format!("* {:.1}", self.title.rating)),
    )
  }
}
impl IntoElement for TitleListItem {
  type Element = Component<Self>;

  fn into_element(self) -> Self::Element {
    Component::new(self)
  }
}

struct TitlesListAdapter {
  sections: Vec<Section>,
}
impl ListAdapter for TitlesListAdapter {
  type Item = TitleListItem;

  fn sections_count(&self) -> usize {
    self.sections.len()
  }

  fn items_count(&mut self, section: usize) -> usize {
    self
      .sections
      .get(section)
      .map_or(0, |section| section.titles.len())
  }

  fn render_item(
    &mut self,
    idx_path: IndexPath,
    _: &mut Window,
    _: &mut Context<ListState<Self>>,
  ) -> Option<Self::Item>
  where
    Self: Sized,
  {
    let title = self
      .sections
      .get(idx_path.section)?
      .titles
      .get(idx_path.row)?;

    Some(TitleListItem::new(title.clone()))
  }

  fn render_header(
    &mut self,
    section_idx: usize,
    _: &mut Window,
    _: &mut Context<ListState<Self>>,
  ) -> Option<impl IntoElement>
  where
    Self: Sized,
  {
    let section = self.sections.get(section_idx)?;

    Some(
      div()
        .w_full()
        .px(1.)
        .bg(rgb(0xff0000))
        .child(format!("{:?}", section.category)),
    )
  }

  fn render_footer(
    &mut self,
    section_idx: usize,
    _: &mut Window,
    _: &mut Context<ListState<Self>>,
  ) -> Option<impl IntoElement>
  where
    Self: Sized,
  {
    let section = self.sections.get(section_idx)?;
    let summary = format!(
      "total {} | avg {:.1}",
      section.titles.len(),
      section.avg_rating(),
    );

    Some(div().w_full().px(1.).bg(rgb(0x0000ff)).child(summary))
  }

  fn render_empty(
    &mut self,
    _: &mut Window,
    _: &mut Context<ListState<Self>>,
  ) -> impl IntoElement
  where
    Self: Sized,
  {
    div()
      .size_full()
      .flex()
      .items_center()
      .justify_center()
      .child("no titles available")
  }

  fn has_header(&self, section: usize) -> bool {
    self.sections.get(section).is_some()
  }
  fn has_footer(&self, section: usize) -> bool {
    self.sections.get(section).is_some()
  }
}
