// SPDX-License-Identifier: Apache-2.0

use std::{
  cell::RefCell,
  rc::{self, Rc},
};

use anyhow::Context as _;

use crate::{
  AnyView, AnyWindowHandle, App, AppContext, BackgroundExecutor, Context,
  Entity, ForegroundExecutor, Window,
};

#[derive(Debug)]
pub struct AsyncApp {
  pub(crate) app: rc::Weak<RefCell<App>>,
  pub(crate) foreground_executor: ForegroundExecutor,
  pub(crate) background_executor: BackgroundExecutor,
}
impl AsyncApp {
  pub fn app(&self) -> Rc<RefCell<App>> {
    self
      .app
      .upgrade()
      .context("app already been dropped")
      .unwrap()
  }
}
impl AppContext for AsyncApp {
  fn new_entity<F, E>(&mut self, f: F) -> Entity<E>
  where
    F: FnOnce(&mut Context<E>) -> E,
    E: 'static,
  {
    let app = self.app();
    let mut app = app.borrow_mut();
    app.new_entity(f)
  }
  fn read_entity<E, F, R>(&self, handle: &Entity<E>, f: F) -> R
  where
    E: 'static,
    F: FnOnce(&E, &App) -> R,
  {
    let app = self.app();
    let app = app.borrow();
    app.read_entity(handle, f)
  }
  fn update_entity<E, F, R>(&mut self, handle: &Entity<E>, f: F) -> R
  where
    E: 'static,
    F: FnOnce(&mut E, &mut Context<E>) -> R,
  {
    let app = self.app();
    let mut app = app.borrow_mut();
    app.update_entity(handle, f)
  }
  fn update_window<F, R>(
    &mut self,
    handle: AnyWindowHandle,
    f: F,
  ) -> anyhow::Result<R>
  where
    F: FnOnce(AnyView, &mut Window, &mut App) -> R,
  {
    let app = self.app();
    let mut app = app.borrow_mut();
    app.update_window_id(handle.window_id, f)
  }

  fn read_global<G, F, R>(&self, f: F) -> R
  where
    G: crate::Global,
    F: FnOnce(&G, &App) -> R,
  {
    let app = self.app();
    let app = app.borrow();
    app.read_global(f)
  }
}
