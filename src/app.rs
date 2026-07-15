// SPDX-License-Identifier: Apache-2.0

use std::{
  any::{Any, TypeId},
  cell::RefCell,
  collections::VecDeque,
  rc::{self, Rc},
  sync::atomic::{self, AtomicBool},
};

use crossterm::event as term_event;
use crossterm::event::EventStream;
use futures_util::StreamExt as _;
use parking_lot::RwLock;
use rustc_hash::FxHashMap;
use slotmap::SlotMap;

use crate::{
  AnyWindowHandle, Entity, EntityId, EntityMap, Global, Keystroke, Render,
  TERM, Terminal, Window, WindowHandle, WindowId, WindowOptions, get_terminal,
};

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
    let rt = tokio::runtime::Handle::try_current();
    let cx = self.app.clone();

    match rt {
      Ok(rt) => {
        if matches!(
          rt.runtime_flavor(),
          tokio::runtime::RuntimeFlavor::CurrentThread
        ) {
          panic!("required runtime flavor is `rt-multi-thread`");
        };

        tokio::task::block_in_place(move || {
          rt.block_on(async move { App::run(cx, f).await })
        })
      }
      Err(..) => {
        let rt = tokio::runtime::Builder::new_multi_thread()
          .enable_all()
          .build()
          .unwrap();
        rt.block_on(async move { App::run(cx, f).await })
      }
    }
  }
}
impl Default for Application {
  fn default() -> Self {
    Self::new()
  }
}

#[derive(Debug)]
pub struct App {
  this: rc::Weak<RefCell<Self>>,
  quitting: AtomicBool,

  globals_by_type: FxHashMap<TypeId, Box<dyn Any>>,

  active_window: Option<AnyWindowHandle>,
  windows: SlotMap<WindowId, Option<Box<Window>>>,

  entities: EntityMap,

  pending_updates: u32,
  pending_effects: VecDeque<Effect>,
  flushing_effects: bool,
}
impl App {
  fn create() -> Rc<RefCell<Self>> {
    TERM
      .set(RwLock::new(Terminal::new()))
      .expect("failed to init terminal");

    Rc::new_cyclic(|this| {
      RefCell::new(Self {
        this: this.clone(),
        quitting: AtomicBool::new(false),
        globals_by_type: Default::default(),
        active_window: None,
        windows: Default::default(),
        entities: Default::default(),
        pending_updates: 0,
        pending_effects: Default::default(),
        flushing_effects: false,
      })
    })
  }

  pub fn open_window<F, V>(
    &mut self,
    window_options: WindowOptions,
    f: F,
  ) -> WindowHandle<V>
  where
    F: FnOnce(&mut Window, &mut Self) -> Entity<V>,
    V: Render,
  {
    self.update(move |cx| {
      let window_id = cx.windows.insert(None);
      let handle = WindowHandle::new(window_id);
      let mut window = Window::new(window_options);
      window.root = Some(f(&mut window, cx).into());
      cx.windows
        .get_mut(window_id)
        .unwrap()
        .replace(Box::new(window));
      cx.active_window = Some(*handle);
      handle
    })
  }

  async fn run<F, R>(this: Rc<RefCell<Self>>, f: F) -> anyhow::Result<R>
  where
    F: FnOnce(&mut Self) -> R,
  {
    let result = f(&mut this.borrow_mut());

    let mut event_stream = EventStream::new();

    while !this.borrow().quitting.load(atomic::Ordering::Relaxed) {
      tokio::select! {
        Some(Ok(event)) = event_stream.next() => {
          this.borrow_mut().handle_event(event);
        }
      }
    }

    anyhow::Ok(result)
  }

  fn handle_key_event(&mut self, key_event: term_event::KeyEvent) {
    use term_event::KeyModifiers;

    let mut keystroke = String::new();

    if matches!(key_event.modifiers, KeyModifiers::SHIFT) {
      keystroke.push_str("shift-");
    };
    if matches!(key_event.modifiers, KeyModifiers::CONTROL) {
      keystroke.push_str("ctrl-");
    };
    if matches!(key_event.modifiers, KeyModifiers::ALT) {
      keystroke.push_str("alt-");
    };
    if matches!(
      key_event.modifiers,
      KeyModifiers::SUPER | KeyModifiers::HYPER | KeyModifiers::META
    ) {
      keystroke.push_str("meta-");
    };

    if let Ok(keystroke) = Keystroke::parse(&keystroke) {};
  }
  fn handle_event(&mut self, event: term_event::Event) {
    match event {
      term_event::Event::Key(key_event) => {
        self.handle_key_event(key_event);
      }
      term_event::Event::Resize(width, height) => {}

      _ => {}
    };
  }

  pub fn global<G>(&self) -> &G
  where
    G: Global,
  {
    self.try_global().unwrap()
  }
  pub fn try_global<G>(&self) -> Option<&G>
  where
    G: Global,
  {
    self
      .globals_by_type
      .get(&TypeId::of::<G>())
      .and_then(|global| global.downcast_ref())
  }
  fn update<F, R>(&mut self, f: F) -> R
  where
    F: FnOnce(&mut Self) -> R,
  {
    self.pending_updates += 1;
    let result = f(self);
    self.finish_update();
    result
  }
  fn finish_update(&mut self) {
    if self.pending_updates == 1 && !self.flushing_effects {
      self.flushing_effects = true;
      self.flush_effects();
      self.flushing_effects = false;
    };
    self.pending_updates -= 1;
  }
  fn flush_effects(&mut self) {
    while let Some(effect) = self.pending_effects.pop_front() {
      // TODO
      #[expect(clippy::match_single_binding)]
      match effect {
        _ => {}
      }
    }
  }
}
impl AppContext for App {
  fn new_entity<F, E>(&mut self, f: F) -> Entity<E>
  where
    E: 'static,
    F: FnOnce(&mut Context<E>) -> E,
  {
    self.update(|cx| {
      let slot = cx.entities.reserve();
      let handle = slot.0.clone();

      let entity = f(&mut Context::new(cx, handle.clone()));
      cx.entities.insert(slot, entity);
      handle
    })
  }

  fn read_entity<E, F, R>(&self, handle: &Entity<E>, f: F) -> R
  where
    E: 'static,
    F: FnOnce(&E, &App) -> R,
  {
    let entity = self.entities.read(handle);
    f(entity, self)
  }
  fn update_entity<E, F, R>(&mut self, handle: &Entity<E>, f: F) -> R
  where
    E: 'static,
    F: FnOnce(&mut E, &mut Context<E>) -> R,
  {
    self.update(|cx| {
      let mut lease = cx.entities.lease(handle);
      let result = f(&mut lease, &mut Context::new(cx, handle.clone()));
      cx.entities.end_lease(lease);
      result
    })
  }

  fn read_global<G, F, R>(&self, f: F) -> R
  where
    G: Global,
    F: FnOnce(&G, &App) -> R,
  {
    let global = self.global();
    f(global, self)
  }
}

#[derive(Debug)]
#[derive(derive_more::Deref, derive_more::DerefMut)]
pub struct Context<'a, E> {
  #[deref]
  #[deref_mut]
  app: &'a mut App,
  entity: Entity<E>,
}
impl<'a, E> Context<'a, E> {
  pub(crate) const fn new(app: &'a mut App, entity: Entity<E>) -> Self {
    Self { app, entity }
  }
}

pub trait AppContext {
  fn new_entity<F, E>(&mut self, f: F) -> Entity<E>
  where
    E: 'static,
    F: FnOnce(&mut Context<E>) -> E;

  fn read_entity<E, F, R>(&self, handle: &Entity<E>, f: F) -> R
  where
    E: 'static,
    F: FnOnce(&E, &App) -> R;
  fn update_entity<E, F, R>(&mut self, handle: &Entity<E>, f: F) -> R
  where
    E: 'static,
    F: FnOnce(&mut E, &mut Context<E>) -> R;

  fn read_global<G, F, R>(&self, f: F) -> R
  where
    G: Global,
    F: FnOnce(&G, &App) -> R;
}

#[derive(Debug)]
enum Effect {
  Emit {
    emitter: EntityId,
    event: Box<dyn Any>,
    event_ty: TypeId,
  },
  Notify {
    entity_id: EntityId,
  },
}
