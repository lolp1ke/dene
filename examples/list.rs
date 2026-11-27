// SPDX-License-Identifier: Apache-2.0

use std::{rc::Rc, sync::Arc};

use dene::{
  app::{App, AppContext, Application, Context},
  color::rgb,
  element::{
    Component, Confirmable, ElementExt, IntoElement, ParentElement, Render,
    RenderOnce, Selectable, StyleableElement,
  },
  elements::{
    IndexPath, List, ListAdapter, ListEvent, ListItem, ListState, div,
  },
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
  status: Arc<str>,
}
impl ListExample {
  fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
    let this = cx.entity();
    let list_state = cx.new_entity(|cx| {
      ListState::new(TitlesListAdapter::sample(), window, cx).selectable(true)
    });

    cx.on_event(&list_state, move |_, event: &ListEvent, cx| {
      let status: Arc<str> = match event {
        ListEvent::Select(idx) => {
          format!("selected: section {}, row {}", idx.section, idx.row).into()
        }
        ListEvent::Submit(idx) => {
          format!("submitted: section {}, row {}", idx.section, idx.row).into()
        }
        ListEvent::Clear => "selection cleared".into(),
      };

      this.update(cx, |this, cx| {
        this.status = status;
        cx.notify();
      });
      true
    });

    Self {
      list_state,
      status: "".into(),
    }
  }
}
impl Render for ListExample {
  fn render(
    &mut self,
    _: &mut Window,
    _: &mut Context<Self>,
  ) -> impl IntoElement {
    div()
      .size_full()
      .flex()
      .flex_col()
      .child(
        "up/down: move | return: submit | esc: clear | ctrl-; ctrl-q: quit",
      )
      .child(
        div()
          .flex_grow(1.)
          .min_h(0.)
          .w_full()
          .border()
          .child(List::new(&self.list_state).separator(true)),
      )
      .child(
        div()
          .w_full()
          .px(1.)
          .bg(rgb(0x161922))
          .child(self.status.clone()),
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
  fn new(title: Rc<Title>, selected: bool, confirmed: bool) -> Self {
    Self {
      base: ListItem::new().selected(selected).confirmed(confirmed),
      title,
    }
  }
}
impl RenderOnce for TitleListItem {
  fn render(self, _: &mut Window, _: &mut App) -> impl IntoElement {
    let selected = self.base.is_selected();
    let confirmed = self.base.is_confirmed();
    let prefix = if selected { "> " } else { "  " };
    let badge = if confirmed { " [confirmed]" } else { "" };

    self
      .base
      .w_full()
      .overflow_x_hidden()
      .when(confirmed && !selected, |this| this.bg(rgb(0x1f3a2e)))
      .when(selected, |this| this.bg(rgb(0x3b4261)))
      .child(
        div()
          .w_full()
          .flex()
          .flex_row()
          .justify_between()
          .px(1.)
          .child(format!(
            "{}{} by {}{}",
            prefix, self.title.name, self.title.author, badge,
          ))
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
  selected_idx: Option<IndexPath>,
  confirmed_idx: Option<IndexPath>,
  loading: bool,
}
impl TitlesListAdapter {
  fn sample() -> Self {
    Self {
      sections: vec![
        Section {
          category: Category::Series,
          titles: vec![
            Title::new("breaking bad", "vince gilligan", 9.5),
            Title::new("andor", "tony gilroy", 8.6),
            Title::new("severance", "dan erickson", 8.7),
            Title::new("the wire", "david simon", 9.3),
          ],
        },
        Section {
          category: Category::Film,
          titles: vec![
            Title::new("spirited away", "hayao miyazaki", 8.6),
            Title::new("parasite", "bong joon-ho", 8.5),
            Title::new("dune: part two", "denis villeneuve", 8.5),
            Title::new("blade runner 2049", "denis villeneuve", 8.0),
          ],
        },
        Section {
          category: Category::Manga,
          titles: vec![
            Title::new("one piece", "eiichiro oda", 10.0),
            Title::new("berserk", "kentaro miura", 9.5),
            Title::new("vagabond", "takehiko inoue", 9.2),
            Title::new("monster", "naoki urasawa", 9.1),
            Title::new("vinland saga", "makoto yukimura", 9.0),
          ],
        },
      ],
      selected_idx: None,
      confirmed_idx: None,
      loading: false,
    }
  }
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

  fn set_selected_idx(
    &mut self,
    idx_path: Option<IndexPath>,
    cx: &mut Context<ListState<Self>>,
  ) {
    self.selected_idx = idx_path;
    cx.notify();
  }

  fn confirm(&mut self, _: &mut Window, cx: &mut Context<ListState<Self>>) {
    self.confirmed_idx = self.selected_idx;
    cx.notify();
  }

  fn cancel(&mut self, _: &mut Window, cx: &mut Context<ListState<Self>>) {
    self.selected_idx = None;
    self.confirmed_idx = None;
    cx.notify();
  }

  fn loading(&self, _: &App) -> bool {
    self.loading
  }

  fn render_loading(
    &mut self,
    _: &mut Window,
    _: &mut Context<ListState<Self>>,
  ) -> Option<impl IntoElement>
  where
    Self: Sized,
  {
    Some(
      div()
        .size_full()
        .flex()
        .items_center()
        .justify_center()
        .child("loading titles..."),
    )
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
    let selected = self
      .selected_idx
      .is_some_and(|current| current.eq_row(idx_path));
    let confirmed = self
      .confirmed_idx
      .is_some_and(|current| current.eq_row(idx_path));
    let title = self
      .sections
      .get(idx_path.section)?
      .titles
      .get(idx_path.row)?;

    Some(TitleListItem::new(title.clone(), selected, confirmed))
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
        .bg(rgb(0x2b3040))
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

    Some(div().w_full().px(1.).bg(rgb(0x1f2330)).child(format!(
      "{} | avg {:.1}",
      section.titles.len(),
      section.avg_rating()
    )))
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
