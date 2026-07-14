// SPDX-License-Identifier: Apache-2.0

use crate::{FocusId, Rect};

slotmap::new_key_type! {
  pub struct WindowId;
}

#[derive(Debug)]
pub struct Window {
  pub(crate) focus: Option<FocusId>,
  pub(crate) rect: Rect,
  pub(crate) dirty: bool,
}
impl Window {
  pub(crate) fn new(opts: WindowOptions) -> Self {
    let WindowOptions { rect } = opts;

    Self {
      focus: None,
      rect,
      dirty: false,
    }
  }
}

#[derive(Debug)]
pub struct WindowOptions {
  rect: Rect,
}
