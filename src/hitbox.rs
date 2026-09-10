// SPDX-License-Identifier: Apache-2.0

use crate::{Pos, Rect, Window};

#[derive(Debug)]
#[derive(Clone)]
pub struct Hitbox {
  pub(crate) bounds: Rect,
}
impl Hitbox {
  pub(crate) fn is_hovered(&self, window: &Window) -> bool {
    self.contains(window.mouse_position)
  }

  pub(crate) fn contains(&self, pos: Pos) -> bool {
    pos.x >= self.bounds.x
      && u32::from(pos.x)
        < u32::from(self.bounds.x) + u32::from(self.bounds.width)
      && pos.y >= self.bounds.y
      && u32::from(pos.y)
        < u32::from(self.bounds.y) + u32::from(self.bounds.height)
  }
}
