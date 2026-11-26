// SPDX-License-Identifier: Apache-2.0

use std::{ops::Range, sync::Arc};

use crate::{App, FocusHandle, Focusable, Keybind, Keystroke, TextAlign};

mod actions {
  use crate::actions;
  actions! {
    "table",
    [
      Left,
      Right,
      Up,
      Down,
      Cancel,
    ]
  }
}
use actions::*;

#[derive(Debug)]
pub enum TableEvent {
  SelectedRow(usize),
  SelectedColumn(usize),
  SelectedCell(usize, usize),
  ClearSelection,
}

#[derive(Debug)]
pub struct TableState {
  focus_handle: FocusHandle,

  selected_row: Option<usize>,
  selection_mode: SelectionMode,

  visible_range: TableVisibleRange,
}
impl Focusable for TableState {
  fn focus_handle(&self, _: &App) -> FocusHandle {
    self.focus_handle.clone()
  }
}

#[derive(Debug)]
#[derive(Default)]
enum SelectionMode {
  #[default]
  Row,
  Column,
  Cell,
}

#[derive(Debug)]
struct TableVisibleRange {
  rows: Range<usize>,
  columns: Range<usize>,
}

#[derive(Debug)]
#[derive(Default)]
pub struct Column {
  pub(crate) key: Arc<str>,
  pub(crate) name: Arc<str>,
  pub(crate) width: u16,
  pub(crate) min_width: u16,
  pub(crate) max_width: u16,
  pub(crate) align: TextAlign,
  pub(crate) sort: Option<ColumnSort>,
  pub(crate) fixed: Option<ColumnFixed>,
  pub(crate) selectable: bool,
}
impl Column {
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
#[derive(Debug)]
pub enum ColumnSort {
  Default,
  Ascending,
  Descending,
}
#[derive(Debug)]
pub enum ColumnFixed {
  Left,
  Right,
}
#[derive(Debug)]
pub struct ColumnGroup {
  pub(crate) label: Arc<str>,
  pub(crate) span: usize,
}
impl ColumnGroup {
  pub fn new<S>(label: S, span: usize) -> Self
  where
    S: Into<Arc<str>>,
  {
    Self {
      label: label.into(),
      span,
    }
  }
}

const KEY_CONTEXT: &str = "table";
pub(crate) fn init(cx: &mut App) {
  let key_context = Some(KEY_CONTEXT);
  cx.bind_keys([
    Keybind::new(Left, [Keystroke::parse("left")], key_context),
    Keybind::new(Right, [Keystroke::parse("right")], key_context),
    Keybind::new(Up, [Keystroke::parse("up")], key_context),
    Keybind::new(Down, [Keystroke::parse("down")], key_context),
    Keybind::new(Cancel, [Keystroke::parse("esc")], key_context),
  ]);
}
