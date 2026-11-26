// SPDX-License-Identifier: Apache-2.0

#[allow(unused)]
mod _table;
mod common;
mod div;
mod empty;
mod input;
mod list;
// mod table;
mod text;
mod uniform_list;

pub use _table::*;
pub use div::*;
pub use empty::*;
pub use input::*;
pub use list::*;
// pub use table::*;
pub use common::*;
pub use text::*;
pub use uniform_list::*;

use crate::App;

pub(crate) fn init(cx: &mut App) {
  input::init(cx);
  _table::init(cx);
  list::init(cx);
  // table::init(cx);
}
