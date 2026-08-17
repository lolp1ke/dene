// SPDX-License-Identifier: Apache-2.0

#[allow(unused)]
mod _table;
mod div;
mod empty;
mod input;
mod list;
mod text;
mod virtual_list;

pub use _table::*;
pub use div::*;
pub use empty::*;
pub use input::*;
pub use list::*;
pub use text::*;
pub use virtual_list::*;

use crate::App;

pub(crate) fn init(cx: &mut App) {
  input::init(cx);
  list::init(cx);
  _table::init(cx);
}
