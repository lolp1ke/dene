// SPDX-License-Identifier: Apache-2.0

use std::{
  cell::RefCell,
  rc::{self, Rc},
  sync::atomic::{self, AtomicBool},
};

use crossterm::event as term_event;
use crossterm::event::EventStream;
use futures_util::StreamExt as _;
use slotmap::SlotMap;

use crate::{Window, WindowId, app};

#[derive(Debug)]
pub struct Application {
  app: Rc<RefCell<App>>,
}
impl Application {
  pub fn new() -> Self {
    Self { app: App::create() }
  }

  pub fn run<F, R>(self, f: F) -> anyhow::Result<R>
  where
    F: FnOnce(&mut App) -> R,
  {
    let rt = tokio::runtime::Handle::current();
    let cx = self.app.clone();

    rt.block_on(async move { App::run(cx, f).await })
  }
}

#[derive(Debug)]
pub struct App {
  this: rc::Weak<RefCell<Self>>,

  windows: SlotMap<WindowId, Option<Box<Window>>>,

  quitting: AtomicBool,
}
impl App {
  fn create() -> Rc<RefCell<Self>> {
    Rc::new_cyclic(|this| {
      RefCell::new(Self {
        this: this.clone(),
        windows: Default::default(),
        quitting: AtomicBool::new(false),
      })
    })
  }

  pub fn open_window(&mut self) {
    let window_id = self.windows.insert(None);
  }

  async fn run<F, R>(this: Rc<RefCell<Self>>, f: F) -> anyhow::Result<R>
  where
    F: FnOnce(&mut Self) -> R,
  {
    let result = f(&mut this.borrow_mut());

    let mut event_stream = EventStream::new();

    while this.borrow().quitting.load(atomic::Ordering::Relaxed) {
      tokio::select! {
        Some(Ok(event)) = event_stream.next() => {
          this.borrow_mut().handle_event(event);
        }
      }
    }

    anyhow::Ok(result)
  }

  fn handle_event(&mut self, event: term_event::Event) {
    match event {
      term_event::Event::Key(key) => {}
      term_event::Event::Resize(width, height) => {}

      _ => {}
    };
  }
}

struct Effect {}
