// SPDX-License-Identifier: Apache-2.0

#[allow(unused)]
mod _table;
mod div;
mod empty;
mod input;
mod text;
mod uniform_list;

pub use _table::*;
pub use div::*;
pub use empty::*;
pub use input::*;
pub use text::*;
pub use uniform_list::*;

use crate::App;

pub(crate) fn init(cx: &mut App) {
  input::init(cx);
  _table::init(cx);
}
