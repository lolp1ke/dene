// SPDX-License-Identifier: Apache-2.0

use crate::{Rect, Window};

#[derive(Debug)]
#[derive(Clone)]
pub struct Hitbox {
  pub(crate) bounds: Rect,
}
impl Hitbox {
  pub(crate) fn is_hovered(&self, window: &Window) -> bool {
    todo!("h");
  }
}
