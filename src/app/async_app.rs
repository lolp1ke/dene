use std::{
  cell::RefCell,
  rc::{self, Rc},
};

use anyhow::Context;

use crate::{App, BackgroundExecutor, ForegroundExecutor};

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
