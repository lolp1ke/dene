// SPDX-License-Identifier: Apache-2.0

use std::{cell::RefCell, rc::Rc};

use crate::Rect;

#[derive(Debug)]
#[derive(Clone)]
pub struct ScrollHandle(pub(crate) Rc<RefCell<ScrollHandleInner>>);

#[derive(Debug)]
pub(crate) struct ScrollHandleInner {
  pub(crate) bounds: Rect,
  pub(crate) offset: Rc<RefCell<[u16; 2]>>,
  pub(crate) overflow: [taffy::Overflow; 2],
}
