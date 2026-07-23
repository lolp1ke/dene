// SPDX-License-Identifier: Apache-2.0

use std::{
  cell::RefCell,
  rc::{self, Rc},
};

use crate::{
  AnyView, AnyWindowHandle, App, AppContext, Context, Entity,
  ForegroundExecutor, Global, Task, Window,
};

#[derive(Debug)]
#[derive(Clone)]
pub struct AsyncApp {
  pub(crate) app: rc::Weak<RefCell<App>>,
  pub(crate) foreground_executor: ForegroundExecutor,
}
impl AsyncApp {
  pub fn app(&self) -> Rc<RefCell<App>> {
    self.app.upgrade().expect("app is already been dropped")
  }

  pub fn spawn<AsyncFn, R>(&self, f: AsyncFn) -> Task<R>
  where
    AsyncFn: 'static + AsyncFnOnce(&mut Self) -> R,
    R: 'static,
  {
    let mut cx = self.clone();
    self
      .foreground_executor
      .spawn(async move { f(&mut cx).await })
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
