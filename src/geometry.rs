// SPDX-License-Identifier: Apache-2.0

#[derive(Debug)]
#[derive(Clone, Copy)]
#[derive(Default)]
pub struct Rect {
  pub(crate) x: u16,
  pub(crate) y: u16,
  pub(crate) width: u16,
  pub(crate) height: u16,
}
impl Rect {
  pub(crate) fn as_size(&self) -> Size {
    Size {
      width: self.width,
      height: self.height,
    }
  }
  pub(crate) fn as_pos(&self) -> Pos {
    Pos {
      x: self.x,
      y: self.y,
    }
  }
  pub(crate) fn as_bottom_right_pos(&self) -> Pos {
    Pos {
      x: self.x + self.width,
      y: self.y + self.height,
    }
  }
}

#[derive(Debug)]
#[derive(Clone, Copy)]
#[derive(PartialEq)]
#[derive(Default)]
pub struct Size {
  pub(crate) width: u16,
  pub(crate) height: u16,
}

#[derive(Debug)]
#[derive(Clone, Copy)]
#[derive(PartialEq)]
#[derive(Default)]
#[derive(
  derive_more::Add,
  derive_more::AddAssign,
  derive_more::Sub,
  derive_more::SubAssign,
  derive_more::Mul,
  derive_more::MulAssign,
  derive_more::Div,
  derive_more::DivAssign
)]
pub struct Pos {
  pub(crate) x: u16,
  pub(crate) y: u16,
}
impl Pos {
  pub(crate) fn min(self, other: Self) -> Self {
    Self {
      x: if self.x <= other.x { self.x } else { other.x },
      y: if self.y <= other.y { self.y } else { other.y },
    }
  }
  pub(crate) fn max(self, other: Self) -> Self {
    Self {
      x: if self.x >= other.x { self.x } else { other.x },
      y: if self.y >= other.y { self.y } else { other.y },
    }
  }
}
