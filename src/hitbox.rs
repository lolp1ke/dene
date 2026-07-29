// SPDX-License-Identifier: Apache-2.0

use crate::{Rect, Window};

#[derive(Debug)]
#[derive(Clone)]
pub struct Hitbox {
  pub(crate) bounds: Rect,
}
impl Hitbox {
  pub(crate) fn is_hovered(&self, window: &Window) -> bool {
    let pos = window.mouse_position;
    pos.x >= self.bounds.x
      && pos.x < self.bounds.x + self.bounds.width
      && pos.y >= self.bounds.y
      && pos.y < self.bounds.y + self.bounds.height
  }
}
