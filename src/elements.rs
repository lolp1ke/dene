// SPDX-License-Identifier: Apache-2.0

mod _list;
mod _table;
mod div;
mod empty;
mod input;
pub mod list;
mod text;

pub use _list::*;
pub use _table::*;
pub use div::*;
pub use empty::*;
pub use input::*;
pub use text::*;

use crate::App;

pub(crate) fn init(cx: &mut App) {
  input::init(cx);
  _list::init(cx);
  _table::init(cx);
}
