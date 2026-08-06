// SPDX-License-Identifier: Apache-2.0

mod div;
mod empty;
mod input;
mod list;
mod table;
mod text;

pub use div::*;
pub use empty::*;
pub use input::*;
pub use list::*;
pub use table::*;
pub use text::*;

use crate::App;

pub(crate) fn init(cx: &mut App) {
  input::init(cx);
  list::init(cx);
  table::init(cx);
}
