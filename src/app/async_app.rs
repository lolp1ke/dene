// SPDX-License-Identifier: Apache-2.0

use std::{
  cell::RefCell,
  rc::{self, Rc},
};

use crate::{
  AnyView, AnyWindowHandle, App, AppContext, Context, Entity, Global, Window,
};

#[derive(Debug)]
pub struct AsyncApp {
  pub(crate) app: rc::Weak<RefCell<App>>,
}
impl AsyncApp {
  fn app(&self) -> Rc<RefCell<App>> {
    self.app.upgrade().expect("app is already been dropped")
  }
}
impl AppContext for AsyncApp {
  fn new_entity<F, E>(&mut self, f: F) -> Entity<E>
  where
    E: 'static,
    F: FnOnce(&mut Context<E>) -> E,
  {
    let cx = self.app();
    cx.borrow_mut().new_entity(f)
  }

  fn read_entity<E, F, R>(&self, handle: &Entity<E>, f: F) -> R
  where
    E: 'static,
    F: FnOnce(&E, &App) -> R,
  {
    let cx = self.app();
    cx.borrow().read_entity(handle, f)
  }
  fn update_entity<E, F, R>(&mut self, handle: &Entity<E>, f: F) -> R
  where
    E: 'static,
    F: FnOnce(&mut E, &mut Context<E>) -> R,
  {
    let cx = self.app();
    cx.borrow_mut().update_entity(handle, f)
  }

  fn update_window<F, R>(
    &mut self,
    handle: AnyWindowHandle,
    f: F,
  ) -> anyhow::Result<R>
  where
    F: FnOnce(AnyView, &mut Window, &mut App) -> R,
  {
    let cx = self.app();
    cx.borrow_mut().update_window(handle, f)
  }

  fn read_global<G, F, R>(&self, f: F) -> R
  where
    G: Global,
    F: FnOnce(&G, &App) -> R,
  {
    let cx = self.app();
    cx.borrow().read_global(f)
  }
}
