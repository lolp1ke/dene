// SPDX-License-Identifier: Apache-2.0

use std::{io::Stdout, sync::OnceLock};

use parking_lot::RwLock;
use ratatui::{
  Frame, Terminal, crossterm, layout::Rect, prelude::CrosstermBackend,
};

use crate::{AnyEntity, App, Context, Entity, EntityId, Keystroke, Window};

#[expect(unused_variables, reason = "default noop implementation")]
pub trait Render: 'static + Sized {
  fn render(
    &mut self,
    frame: &mut Frame,
    area: Rect,
    window: &mut Window,
    cx: &mut Context<Self>,
  ) {
  }
}
#[expect(unused_variables, reason = "default noop implementation")]
pub trait Interactive: 'static + Sized {
  fn on_keystroke(
    &mut self,
    keystroke: Keystroke,
    window: &mut Window,
    cx: &mut Context<Self>,
  ) {
  }
}

#[derive(Debug)]
#[derive(Clone)]
pub struct AnyView {
  entity: AnyEntity,
  pub render: fn(&Self, &mut Frame, Rect, &mut Window, &mut App),
  pub on_keystroke: fn(&Self, Keystroke, &mut Window, &mut App),
}
impl AnyView {
  pub fn downcast<E>(self) -> Option<Entity<E>>
  where
    E: 'static,
  {
    self.entity.downcast()
  }

  pub fn entity_id(&self) -> EntityId {
    self.entity.entity_id
  }
}
impl<V> From<Entity<V>> for AnyView
where
  V: Render + Interactive,
{
  fn from(value: Entity<V>) -> Self {
    Self {
      entity: value.into(),
      render: render::<V>,
      on_keystroke: on_keystroke::<V>,
    }
  }
}

fn render<V>(
  any_view: &AnyView,
  frame: &mut Frame,
  area: Rect,
  window: &mut Window,
  cx: &mut App,
) where
  V: 'static + Render,
{
  let view = any_view.clone().downcast::<V>().unwrap().clone();
  view.update(cx, |view, cx| {
    view.render(frame, area, window, cx);
  });
}
fn on_keystroke<V>(
  any_view: &AnyView,
  keystroke: Keystroke,
  window: &mut Window,
  cx: &mut App,
) where
  V: 'static + Interactive,
{
  let view = any_view.clone().downcast::<V>().unwrap();
  view.update(cx, |view, cx| view.on_keystroke(keystroke, window, cx));
}

static TERM: OnceLock<RwLock<Terminal<CrosstermBackend<Stdout>>>> =
  OnceLock::new();

pub(crate) fn init_term() {
  let term = ratatui::init();
  crossterm::execute!(
    std::io::stdout(),
    crossterm::event::PushKeyboardEnhancementFlags(
      crossterm::event::KeyboardEnhancementFlags::DISAMBIGUATE_ESCAPE_CODES
    )
  )
  .unwrap();

  TERM.set(RwLock::new(term)).unwrap();
}

pub(crate) fn draw<F, R>(f: F) -> R
where
  F: FnOnce(&mut Frame) -> R,
{
  let terminal = TERM.get().unwrap();
  let mut terminal = terminal.write();
  let mut result = None;
  terminal
    .draw(|frame| {
      result = Some(f(frame));
    })
    .unwrap();
  result.unwrap()
}
pub(crate) fn clamp_area(area: Rect, bounds: Rect) -> Rect {
  let x = area.x.min(bounds.x + bounds.width.saturating_sub(1));
  let y = area.y.min(bounds.y + bounds.height.saturating_sub(1));
  let width = area.width.min(bounds.x + bounds.width - x);
  let height = area.height.min(bounds.y + bounds.height - y);
  Rect {
    x,
    y,
    width,
    height,
  }
}
