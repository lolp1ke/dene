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

  pub(crate) fn scroll_by(&self, dx: i32, dy: i32) {
    let lock = self.0.borrow();
    let mut offset = lock.offset.borrow_mut();
    if matches!(lock.overflow[0], taffy::Overflow::Scroll) {
      let viewport = lock.bounds.width as i32;
      let content = lock.content_size.width as i32;
      let max = (content - viewport).max(0);
      offset.x = (offset.x as i32 + dx).clamp(0, max) as u16;
    }
    if matches!(lock.overflow[1], taffy::Overflow::Scroll) {
      let viewport = lock.bounds.height as i32;
      let content = lock.content_size.height as i32;
      let max = (content - viewport).max(0);
      offset.y = (offset.y as i32 + dy).clamp(0, max) as u16;
    }
  }
  pub(crate) fn scroll_to(&self, x: u16, y: u16) {
    let lock = self.0.borrow();
    let mut offset = lock.offset.borrow_mut();
    if matches!(lock.overflow[0], taffy::Overflow::Scroll) {
      let viewport = lock.bounds.width as i32;
      let content = lock.content_size.width as i32;
      let max = (content - viewport).max(0) as u16;
      offset.x = x.min(max);
    };
    if matches!(lock.overflow[1], taffy::Overflow::Scroll) {
      let viewport = lock.bounds.height as i32;
      let content = lock.content_size.height as i32;
      let max = (content - viewport).max(0) as u16;
      offset.y = y.min(max);
    };
  }

  pub(crate) fn offset(&self) -> Pos {
    *self.0.borrow().offset.borrow()
  }
}

#[derive(Debug)]
pub(crate) struct ScrollHandleInner {
  pub(crate) bounds: Rect,
  pub(crate) offset: Rc<RefCell<Pos>>,
  pub(crate) overflow: [taffy::Overflow; 2],
  pub(crate) content_size: Size,
}
