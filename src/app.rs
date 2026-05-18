// SPDX-License-Identifier: Apache-2.0

mod async_app;

use std::{
  any::{Any, TypeId},
  cell::RefCell,
  collections::VecDeque,
  rc::{self, Rc},
  sync::atomic::{self, AtomicBool},
};

use anyhow::Context as _;
pub use async_app::*;
use futures_util::{FutureExt, StreamExt};
use ratatui::{
  crossterm::event::{self as term_event, EventStream, KeyModifiers},
  layout::Size,
};
use rustc_hash::FxHashMap;
use slotmap::SlotMap;
use tokio::sync::mpsc;

use crate::{
  AnyView, AnyWindowHandle, Entity, EntityId, EntityMap, EventEmitter,
  ForegroundTask, Global, Interactive, Keybind, Keybinds, Keystroke, Panel,
  PanelNode, Render, SubscribtionSet, Task, Window, WindowConfig, WindowHandle,
  WindowId,
  action::{self, Action, ActionRegistry},
  executor::{BackgroundExecutor, ForegroundExecutor},
  init_term,
};

pub(crate) type GlobalActionListener =
  Rc<dyn 'static + Fn(&dyn Action, &mut App)>;
pub(crate) type EventListener =
  Box<dyn 'static + FnMut(&dyn Any, &mut App) -> bool>;

#[derive(derive_more::Debug)]
pub struct App {
  this: rc::Weak<RefCell<Self>>,
  quitting: AtomicBool,

  foreground_executor: ForegroundExecutor,
  background_executor: BackgroundExecutor,

  globals_by_type: FxHashMap<TypeId, Box<dyn Any>>,

  pub(crate) actions: Rc<ActionRegistry>,
  keybinds: Rc<RefCell<Keybinds>>,
  #[debug(skip)]
  pub(crate) global_action_listeners:
    FxHashMap<TypeId, Vec<GlobalActionListener>>,

  windows: SlotMap<WindowId, Option<Box<Window>>>,
  active_window: Option<AnyWindowHandle>,
  #[debug(skip)]
  event_listeners: SubscribtionSet<EntityId, (TypeId, EventListener)>,

  pub(crate) entities: EntityMap,

  pending_updates: usize,
  pending_effects: VecDeque<Effect>,
  flushing_effects: bool,
}
impl App {
  pub fn new(
    foreground_executor: ForegroundExecutor,
    background_executor: BackgroundExecutor,
  ) -> Rc<RefCell<Self>> {
    init_term();

    Rc::new_cyclic(|this| {
      RefCell::new(Self {
        this: this.clone(),
        quitting: AtomicBool::new(false),
        foreground_executor,
        background_executor,
        globals_by_type: FxHashMap::default(),
        actions: Rc::default(),
        keybinds: Rc::default(),
        global_action_listeners: FxHashMap::default(),
        windows: SlotMap::default(),
        active_window: None,
        event_listeners: SubscribtionSet::default(),
        entities: EntityMap::default(),
        pending_updates: 0,
        pending_effects: VecDeque::default(),
        flushing_effects: false,
      })
    })
  }
  pub fn run<F, R>(
    app: Rc<RefCell<Self>>,
    mut foreground_rx: mpsc::UnboundedReceiver<ForegroundTask>,
    f: F,
  ) -> impl Future<Output = anyhow::Result<R>>
  where
    F: FnOnce(&mut Self) -> R,
  {
    let result = f(&mut app.borrow_mut());
    app.borrow_mut().on_action(move |_: &action::Quit, cx| {
      cx.quitting.store(true, atomic::Ordering::Relaxed);
    });

    async move {
      let mut event_reader = EventStream::new();
      let mut tick =
        tokio::time::interval(tokio::time::Duration::from_secs_f64(1.0 / 24.0));

      while !app.borrow().quitting.load(atomic::Ordering::Relaxed) {
        tokio::select! {
          Some(Ok(event)) = event_reader.next() => {
            app.borrow_mut().handle_event(event)?;
          }
          Some(runnable) = foreground_rx.recv() => {
            runnable();
          }

          // TODO: render only on change detected, like: [`.notify()`]
          _ = tick.tick() => {
            let mut windows = std::mem::take(&mut app.borrow_mut().windows);
            for window in windows.iter_mut().flat_map(|(_, window)| window) {
              window.render(&mut app.borrow_mut());
            }
            app.borrow_mut().windows = windows;
          }
        }
      }

      app.borrow_mut().shutdown();
      Ok(result)
    }
  }

  fn shutdown(&mut self) {
    ratatui::restore();
  }

  pub fn open_window<F, V>(
    &mut self,
    window_config: WindowConfig,
    f: F,
  ) -> WindowHandle<V>
  where
    F: 'static + FnOnce(&mut Window, &mut Self) -> Entity<V>,
    V: 'static + Render + Interactive,
  {
    self.update(|cx| {
      let window_id = cx.windows.insert(None);
      let handle = WindowHandle::new(window_id);
      let mut window = Window::new(handle.into(), window_config);

      // build the entity
      let root_view = f(&mut window, cx);
      if window.root.is_none() {
        let pane_id = window.next_pane_id();
        window.root = Some(PanelNode::Leaf(Panel {
          id: pane_id,
          view: root_view.into(),
        }));
        window.active_pane = Some(pane_id);
      };

      // render first frame on creation
      window.render(cx);

      cx.windows
        .get_mut(window_id)
        .unwrap()
        .replace(Box::new(window));
      cx.active_window = Some(*handle);
      handle
    })
  }
  fn update_window_id<F, R>(&mut self, id: WindowId, f: F) -> anyhow::Result<R>
  where
    F: FnOnce(AnyView, &mut Window, &mut App) -> R,
  {
    self
      .update(|cx| {
        let mut window = cx.windows.get_mut(id)?.take()?;
        let view = window
          .root
          .as_ref()?
          .find(window.active_pane?)
          .map(|panel| panel.view.clone())?;

        let result = f(view, &mut window, cx);
        cx.windows.get_mut(id)?.replace(window);

        Some(result)
      })
      .context("window not found")
  }

  fn handle_key_event(
    &mut self,
    key: term_event::KeyEvent,
  ) -> anyhow::Result<()> {
    let mut keystroke = String::new();

    if matches!(key.modifiers, KeyModifiers::SHIFT) {
      keystroke.push_str("shift-");
    };
    if matches!(key.modifiers, KeyModifiers::CONTROL) {
      keystroke.push_str("ctrl-");
    };
    if matches!(key.modifiers, KeyModifiers::ALT) {
      keystroke.push_str("alt-");
    };
    if matches!(
      key.modifiers,
      KeyModifiers::META | KeyModifiers::SUPER | KeyModifiers::HYPER
    ) {
      keystroke.push_str("meta-");
    };
    keystroke.push_str(&key.code.to_string());

    if let Ok(keystroke) = Keystroke::parse(&keystroke) {
      let keybinds = self.keybinds.clone();

      for keybind in keybinds.borrow().iter() {
        if let Some(keystroke1) = keybind.keystrokes.first()
          && *keystroke1 == keystroke
        {
          self.dispatch_action(&*keybind.action);
        };
      }
      // TODO: save for second keybind if no action
      //       e.g: cmd+k cmd+l

      if let Some(active_window) = self.active_window {
        active_window.update(self, |_, window, cx| {
          window.dispatch_keystroke(keystroke, cx);
        })?;
      };
    };

    Ok(())
  }
  fn handle_event(&mut self, event: term_event::Event) -> anyhow::Result<()> {
    match event {
      term_event::Event::Key(key) => {
        self.handle_key_event(key)?;
      }
      term_event::Event::Resize(width, height) => {
        for (_, window) in self.windows.iter_mut() {
          if let Some(window) = window {
            window.bounds = window.bounds.resize(Size::new(width, height));
          };
        }
      }

      _ => {}
    };
    Ok(())
  }

  fn on_action<F, A>(&mut self, listener: F)
  where
    F: 'static + Fn(&A, &mut Self),
    A: Action,
  {
    self
      .global_action_listeners
      .entry(TypeId::of::<A>())
      .or_default()
      .push(Rc::new(move |action, cx| {
        let action = action.as_any().downcast_ref().unwrap();
        (listener)(action, cx);
      }));
  }
  fn dispatch_action(&mut self, action: &dyn Action) {
    if let Some(active_window) = self.active_window {
      if let Err(err) = active_window.update(self, |_, window, cx| {
        window.dispatch_action(action, cx);
      }) {
        tracing::warn!("window update error: {:?}", err)
      };
    } else {
      self.dispatch_global_action_listener(action);
    };
  }
  fn dispatch_global_action_listener(&mut self, action: &dyn Action) {
    let action_ty = action.type_id();
    if let Some(listeners) = self.global_action_listeners.remove(&action_ty) {
      for listener in listeners.iter() {
        (listener)(action, self)
      }

      self.global_action_listeners.insert(action_ty, listeners);
    };
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
    if !self.flushing_effects && self.pending_updates == 1 {
      self.flushing_effects = true;
      self.flush_effects();
      self.flushing_effects = false;
    };
    self.pending_updates -= 1;
  }
  fn flush_effects(&mut self) {
    while let Some(effect) = self.pending_effects.pop_front() {
      match effect {
        Effect::Emit {
          emitter,
          event_ty,
          event,
        } => {
          self.apply_emit(emitter, event_ty, &*event);
        }
      };
    }
  }
  fn apply_emit(
    &mut self,
    emitter: EntityId,
    event_ty: TypeId,
    event: &dyn Any,
  ) {
    self
      .event_listeners
      .clone()
      .retain(emitter, |(_event_ty, cb)| {
        if *_event_ty == event_ty {
          cb(event, self)
        } else {
          true
        }
      });
  }

  pub fn subscribe<E, F, Event>(&mut self, entity: Entity<E>, mut on_event: F)
  where
    E: 'static + EventEmitter<Event>,
    F: 'static + FnMut(Entity<E>, &Event, &mut App),
    Event: 'static,
  {
    self.event_listeners.insert(
      entity.id(),
      (
        TypeId::of::<Event>(),
        Box::new(move |event, cx| {
          let event = event.downcast_ref().expect("wrong event type");
          on_event(entity.clone(), event, cx);
          true
        }),
      ),
    );
  }

  pub fn to_async(&self) -> AsyncApp {
    AsyncApp {
      app: self.this.clone(),
      foreground_executor: self.foreground_executor.clone(),
      background_executor: self.background_executor.clone(),
    }
  }
  pub fn spawn<AsyncFn, R>(&self, f: AsyncFn) -> Task<R>
  where
    AsyncFn: 'static + AsyncFnOnce(&mut AsyncApp) -> R,
    R: 'static,
  {
    let mut cx = self.to_async();
    self
      .foreground_executor
      .spawn(async move { f(&mut cx).await }.boxed_local())
  }
  pub fn spawn_on_background<Fut, R>(&self, f: Fut) -> Task<R>
  where
    Fut: 'static + Future<Output = R> + Send,
    Fut::Output: Send,
    R: 'static,
  {
    self.background_executor.spawn(f)
  }

  pub fn set_global<G>(&mut self, global: G)
  where
    G: Global,
  {
    self
      .globals_by_type
      .insert(TypeId::of::<G>(), Box::new(global));
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
  pub fn global_mut<G>(&mut self) -> &mut G
  where
    G: Global,
  {
    self.try_global_mut().unwrap()
  }
  pub fn try_global_mut<G>(&mut self) -> Option<&mut G>
  where
    G: Global,
  {
    self
      .globals_by_type
      .get_mut(&TypeId::of::<G>())
      .and_then(|global| global.downcast_mut())
  }

  pub fn load_keybinds(&mut self, keybinds: Keybinds) {
    self.bind_keys(keybinds.0);
  }
  fn bind_keys<I>(&mut self, keybinds: I)
  where
    I: IntoIterator<Item = Keybind>,
  {
    let mut lock = self.keybinds.borrow_mut();
    lock.add_bindings(keybinds);
  }
}
impl AppContext for App {
  fn new_entity<F, E>(&mut self, f: F) -> Entity<E>
  where
    F: FnOnce(&mut Context<E>) -> E,
    E: 'static,
  {
    self.update(|cx| {
      let slot = cx.entities.reserve();
      let handle = slot.clone();
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

  fn update_window<F, R>(
    &mut self,
    handle: AnyWindowHandle,
    f: F,
  ) -> anyhow::Result<R>
  where
    F: FnOnce(AnyView, &mut Window, &mut App) -> R,
  {
    self.update_window_id(handle.window_id, f)
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
impl Drop for App {
  fn drop(&mut self) {
    self.shutdown();
  }
}

#[derive(derive_more::Deref, derive_more::DerefMut)]
pub struct Context<'a, E> {
  #[deref]
  #[deref_mut]
  app: &'a mut App,
  entity: Entity<E>,
}
impl<'a, E> Context<'a, E> {
  pub(crate) fn new(app: &'a mut App, entity: Entity<E>) -> Self {
    Self { app, entity }
  }
  pub fn entity(&self) -> Entity<E> {
    self.entity.clone()
  }

  pub fn emit<Event>(&mut self, event: Event)
  where
    Event: 'static,
  {
    self.app.pending_effects.push_back(Effect::Emit {
      emitter: self.entity.id(),
      event_ty: TypeId::of::<Event>(),
      event: Box::new(event),
    });
  }
  pub fn listener<F, A>(
    &self,
    f: F,
  ) -> impl 'static + Fn(&A, &mut Window, &mut App)
  where
    F: 'static + Fn(&mut E, &A, &mut Window, &mut Context<E>),
    E: 'static,
    A: ?Sized,
  {
    let view = self.entity();
    move |e: &A, window: &mut Window, cx: &mut App| {
      view.update(cx, |view, cx| f(view, e, window, cx))
    }
  }
}

pub trait AppContext {
  fn new_entity<F, E>(&mut self, f: F) -> Entity<E>
  where
    F: FnOnce(&mut Context<E>) -> E,
    E: 'static;

  fn read_entity<E, F, R>(&self, handle: &Entity<E>, f: F) -> R
  where
    E: 'static,
    F: FnOnce(&E, &App) -> R;
  fn update_entity<E, F, R>(&mut self, handle: &Entity<E>, f: F) -> R
  where
    E: 'static,
    F: FnOnce(&mut E, &mut Context<E>) -> R;

  fn update_window<F, R>(
    &mut self,
    handle: AnyWindowHandle,
    f: F,
  ) -> anyhow::Result<R>
  where
    F: FnOnce(AnyView, &mut Window, &mut App) -> R;

  fn read_global<G, F, R>(&self, f: F) -> R
  where
    G: Global,
    F: FnOnce(&G, &App) -> R;
}
#[derive(Debug)]
pub(crate) enum Effect {
  Emit {
    emitter: EntityId,
    event_ty: TypeId,
    event: Box<dyn Any>,
  },
}
