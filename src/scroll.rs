// SPDX-License-Identifier: Apache-2.0

use std::{cell::RefCell, rc::Rc};

use crate::{Pos, Rect, Size};

#[derive(Debug)]
#[derive(Clone)]
pub struct ScrollHandle(pub(crate) Rc<RefCell<ScrollHandleInner>>);
impl ScrollHandle {
  pub(crate) fn new(overflow: [taffy::Overflow; 2]) -> Self {
    Self(Rc::new(RefCell::new(ScrollHandleInner {
      bounds: Default::default(),
      offset: Default::default(),
      overflow,
      content_size: Default::default(),
    })))
  }
}

#[derive(Debug)]
pub(crate) struct ScrollHandleInner {
  pub(crate) bounds: Rect,
  pub(crate) offset: Rc<RefCell<Pos>>,
  pub(crate) overflow: [taffy::Overflow; 2],
  pub(crate) content_size: Size,
}
