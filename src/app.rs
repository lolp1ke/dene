// SPDX-License-Identifier: Apache-2.0

use std::{
  any::{Any, TypeId},
  cell::RefCell,
  collections::VecDeque,
  rc::{self, Rc},
  sync::{
    Arc,
    atomic::{self, AtomicBool},
  },
};

use anyhow::Context as _;
use crossterm::event as term_event;
use crossterm::event::EventStream;
use futures_util::{FutureExt as _, StreamExt as _};
use parking_lot::RwLock;
use rustc_hash::FxHashMap;
use slotmap::SlotMap;
use smallvec::smallvec;
use tokio::sync::mpsc::{UnboundedReceiver, unbounded_channel};

use crate::{
  Action, ActionRegistry, AnyView, AnyWindowHandle, BackgroundExecutor,
  DeneInput, DispatchPhase, Entity, EntityId, EntityMap, EventDispatcherSet,
  FocusHandle, FocusMap, FocusNext, FocusPrev, ForegroundExecutor,
  ForegroundTask, Global, KeyDownEvent, KeyUpEvent, Keybind, Keybinds,
  Keystroke, Quit, Render, TERM, Task, Terminal, Window, WindowHandle,
  WindowId, WindowOptions, get_terminal,
};

mod async_app;
pub use self::async_app::*;

#[derive(Debug)]
pub struct Application {
  app: Rc<RefCell<App>>,
  frx: Option<UnboundedReceiver<ForegroundTask>>,
}
impl Application {
  pub fn new() -> Self {
    let (app, frx) = App::create();
    Self {
      app,
      frx: Some(frx),
    }
  }

  pub fn run<F, R>(&mut self, f: F) -> anyhow::Result<R>
  where
    F: FnOnce(&mut App) -> R,
  {
    let rt = tokio::runtime::Handle::try_current();
    let cx = self.app.clone();
    let frx = self.frx.take().unwrap();

    match rt {
      Ok(rt) => {
        if matches!(
          rt.runtime_flavor(),
          tokio::runtime::RuntimeFlavor::CurrentThread
        ) {
          panic!("required runtime flavor is `rt-multi-thread`");
        };

        let handle = Arc::new(rt);
        self
          .app
          .borrow_mut()
          .background_executor
          .pass_handle(handle.clone());
        tokio::task::block_in_place(move || {
          handle.block_on(async move { App::run(cx, frx, f).await })
        })
      }
      Err(..) => {
        let rt = tokio::runtime::Builder::new_multi_thread()
          .enable_all()
          .build()
          .unwrap();
        rt.block_on(async move { App::run(cx, frx, f).await })
      }
    }
  }
}
impl Default for Application {
  fn default() -> Self {
    Self::new()
  }
}

type GlobalActionListener = Rc<dyn Fn(&dyn Any, DispatchPhase, &mut App)>;
type EventDispatcListener = Box<dyn FnMut(&dyn Any, &mut App) -> bool>;

#[derive(derive_more::Debug)]
pub struct App {
  this: rc::Weak<RefCell<Self>>,
  quitting: AtomicBool,

  pub(crate) foreground_executor: ForegroundExecutor,
  pub(crate) background_executor: BackgroundExecutor,

  pub(crate) actions: Rc<ActionRegistry>,
  pub(crate) keybinds: Rc<RefCell<Keybinds>>,
  globals_by_type: FxHashMap<TypeId, Box<dyn Any>>,

  focus_map: FocusMap,
  active_window: Option<AnyWindowHandle>,
  windows: SlotMap<WindowId, Option<Box<Window>>>,

  #[debug(skip)]
  pub(crate) global_action_listeners:
    FxHashMap<TypeId, Vec<GlobalActionListener>>,
  #[debug(skip)]
  pub(crate) event_dispatchers:
    EventDispatcherSet<EntityId, (TypeId, EventDispatcListener)>,
  pub(crate) propagate_event: bool,

  pub(crate) entities: EntityMap,

  pending_updates: u32,
  pending_effects: VecDeque<Effect>,
  flushing_effects: bool,
}
impl App {
  fn create() -> (Rc<RefCell<Self>>, UnboundedReceiver<ForegroundTask>) {
    TERM
      .set(RwLock::new(Terminal::new()))
      .expect("failed to init terminal");
    crate::init_tracing();

    let mut keybinds = Keybinds(Vec::new());
    keybinds.extend([
      Keybind {
        action: Box::new(Quit),
        keystrokes: smallvec![
          Keystroke::parse("ctrl-;").unwrap(),
          Keystroke::parse("ctrl-q").unwrap()
        ],
        key_context: None,
      },
      Keybind {
        action: Box::new(FocusNext),
        keystrokes: smallvec![Keystroke::parse("tab").unwrap()],
        key_context: None,
      },
      Keybind {
        action: Box::new(FocusPrev),
        keystrokes: smallvec![Keystroke::parse("shift-tab").unwrap()],
        key_context: None,
      },
    ]);

    let (tx, rx) = unbounded_channel();

    (
      Rc::new_cyclic(|this| {
        RefCell::new(Self {
          this: this.clone(),
          quitting: AtomicBool::new(false),
          foreground_executor: ForegroundExecutor::new(tx),
          background_executor: BackgroundExecutor::new(),
          actions: Default::default(),
          keybinds: Rc::new(RefCell::new(keybinds)),
          globals_by_type: Default::default(),
          focus_map: Default::default(),
          active_window: None,
          windows: Default::default(),
          global_action_listeners: Default::default(),
          event_dispatchers: Default::default(),
          propagate_event: true,
          entities: Default::default(),
          pending_updates: 0,
          pending_effects: Default::default(),
          flushing_effects: false,
        })
      }),
      rx,
    )
  }

  pub fn bind_key(&mut self, keybind: Keybind) {
    self.keybinds.borrow_mut().0.push(keybind);
  }
  pub fn bind_keys<I>(&mut self, keybinds: I)
  where
    I: IntoIterator<Item = Keybind>,
  {
    self.keybinds.borrow_mut().0.extend(keybinds);
  }

  pub fn to_async(&self) -> AsyncApp {
    AsyncApp {
      app: self.this.clone(),
    }
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

      let mut window = Window::new(window_options, cx);
      window.root = Some(f(&mut window, cx).into());
      window.render(cx);

      cx.windows
        .get_mut(window_id)
        .unwrap()
        .replace(Box::new(window));
      cx.active_window = Some(*handle);
      handle
    })
  }

  async fn run<F, R>(
    this: Rc<RefCell<Self>>,
    mut frx: UnboundedReceiver<ForegroundTask>,
    f: F,
  ) -> anyhow::Result<R>
  where
    F: FnOnce(&mut Self) -> R,
  {
    let result = f(&mut this.borrow_mut());
    this.borrow_mut().on_action(|_: &Quit, _, cx| {
      cx.quitting.store(true, atomic::Ordering::Relaxed);
    });

    let mut event_stream = EventStream::new();

    while !this.borrow().quitting.load(atomic::Ordering::Relaxed) {
      tokio::select! {
        Some(Ok(event)) = event_stream.next() => {
          this.borrow_mut().handle_event(event);
        }
        Some(runnable) = frx.recv() => {
          (runnable)();
          // TODO: this is a workaround
          let handle = this.borrow().active_window;
          if let Some(active_window) = handle {
            _ = active_window.update(&mut *this.borrow_mut(), |_, window, cx| {
              if window.dirty {
                window.render(cx);
              };
            });
          };
        }
      }
    }

    get_terminal().write().restore();
    anyhow::Ok(result)
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
    Fut::Output: 'static,
    R: 'static + Send,
  {
    self.background_executor.spawn(f)
  }

  fn on_action<F, A>(&mut self, listener: F)
  where
    F: 'static + Fn(&A, DispatchPhase, &mut Self),
    A: Action,
  {
    self
      .global_action_listeners
      .entry(TypeId::of::<A>())
      .or_default()
      .push(Rc::new(move |action, phase, cx| {
        if let Some(action) = action.downcast_ref() {
          (listener)(action, phase, cx)
        };
      }));
  }

  fn handle_key_event(&mut self, key_event: term_event::KeyEvent) {
    use term_event::KeyModifiers;

    let mut keystroke = String::new();

    if matches!(
      key_event.code,
      term_event::KeyCode::Char('\0') | term_event::KeyCode::Null
    ) {
      return;
    };

    if key_event.modifiers.contains(KeyModifiers::SHIFT) {
      keystroke.push_str("shift-");
    };
    if key_event.modifiers.contains(KeyModifiers::CONTROL) {
      keystroke.push_str("ctrl-");
    };
    if key_event.modifiers.contains(KeyModifiers::ALT) {
      keystroke.push_str("alt-");
    };
    if key_event.modifiers.intersects(
      KeyModifiers::SUPER | KeyModifiers::HYPER | KeyModifiers::META,
    ) {
      keystroke.push_str("meta-");
    };

    match key_event.code {
      term_event::KeyCode::Backspace => keystroke.push_str("delete"),
      term_event::KeyCode::Enter => keystroke.push_str("return"),
      term_event::KeyCode::Left => keystroke.push_str("left"),
      term_event::KeyCode::Right => keystroke.push_str("right"),
      term_event::KeyCode::Up => keystroke.push_str("up"),
      term_event::KeyCode::Down => keystroke.push_str("down"),
      term_event::KeyCode::Tab => keystroke.push_str("tab"),
      term_event::KeyCode::BackTab => keystroke.push_str("tab"),
      term_event::KeyCode::Delete => keystroke.push_str("delete"),
      term_event::KeyCode::F(f) => {
        keystroke.push('f');
        if f < 10 {
          keystroke.push((f + 48) as char);
        } else {
          // NOTE: won't work on f > 99
          //       keystroke.push('1'); // mb hardcode it? who has fn19 and greator anyway?
          keystroke.push((f / 9 + 48) as char);
          keystroke.push((f - 10 + 48) as char);
        };
      }
      term_event::KeyCode::Char(ch) => {
        keystroke.push(ch);
      }
      code if !key_event.modifiers.intersects(KeyModifiers::all()) => {
        keystroke.push_str(code.to_string().to_ascii_lowercase().as_str());
      }
      _ => {
        if let Some(ch) = key_event.code.as_char() {
          keystroke.push(ch);
        };

        if !key_event.modifiers.is_empty() {
          return;
        };
      }
    }

    if let Ok(keystroke) = Keystroke::parse(&keystroke) {
      let dene_input = match key_event.kind {
        term_event::KeyEventKind::Press => DeneInput::KeyDown(KeyDownEvent {
          keystroke,
          is_held: false,
        }),
        term_event::KeyEventKind::Repeat => DeneInput::KeyDown(KeyDownEvent {
          keystroke,
          is_held: true,
        }),
        term_event::KeyEventKind::Release => {
          DeneInput::KeyUp(KeyUpEvent { keystroke })
        }
      };

      if let Some(active_window) = self.active_window
        && let Some(keyboard_event) = dene_input.keyboard_event()
      {
        _ = active_window.update(self, |_, window, cx| {
          window.dispatch_keyboard_event(keyboard_event, cx);
        });
      };
    };
  }
  fn handle_event(&mut self, event: term_event::Event) {
    match event {
      term_event::Event::Key(key_event) => {
        self.handle_key_event(key_event);
      }
      term_event::Event::Resize(width, height) => {
        if let Some(active_window) = self.active_window {
          _ = active_window.update(self, |_, window, _| {
            window.bounds.width = width;
            window.bounds.height = height;
          });
        };
      }

      _ => {}
    };
  }

  pub(crate) fn dispatch_global_action(&mut self, action: &dyn Action) {
    let action_ty_id = action.as_any().type_id();
    if let Some(global_action_listeners) =
      self.global_action_listeners.remove(&action_ty_id)
    {
      for listener in global_action_listeners.iter() {
        (listener)(action, DispatchPhase::Capture, self);
      }

      // TODO: prevent event propogation if set so
      for listener in global_action_listeners.iter().rev() {
        (listener)(action, DispatchPhase::Bubble, self);
      }

      self
        .global_action_listeners
        .insert(action_ty_id, global_action_listeners);
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

  fn update_window_id<F, R>(
    &mut self,
    window_id: WindowId,
    f: F,
  ) -> anyhow::Result<R>
  where
    F: FnOnce(AnyView, &mut Window, &mut Self) -> R,
  {
    self
      .update(move |cx| {
        let mut window = cx.windows.get_mut(window_id)?.take()?;
        let view = window.root.as_ref().cloned()?;

        let result = f(view, &mut window, cx);
        window.dirty = true;
        cx.windows.get_mut(window_id)?.replace(window);

        Some(result)
      })
      .context("no window id found")
  }

  pub fn on_event<E, F, Event>(&mut self, entity: Entity<E>, mut on_event: F)
  where
    E: 'static,
    F: 'static + FnMut(Entity<E>, &dyn Any, &mut App) -> bool,
    Event: 'static,
  {
    self.event_dispatchers.insert(
      entity.id(),
      (
        TypeId::of::<Event>(),
        Box::new(move |event, cx| {
          if let Some(event) = event.downcast_ref::<Event>() {
            (on_event)(entity.clone(), event, cx)
          } else {
            // TODO: add tracing plz future me
            dbg!("WARN: failed to downcast event type");
            false
          }
        }),
      ),
    );
  }

  pub fn notify(&mut self, entity_id: EntityId) {
    if let Some(active_window) = self.active_window {
      _ = active_window.update(self, |_, window, _| {
        window.dirty = true;
      });
    };
    self.pending_effects.push_back(Effect::Notify { entity_id });
  }
  pub fn apply_emit(
    &mut self,
    emitter: &EntityId,
    event_ty: TypeId,
    event: &dyn Any,
  ) {
    self
      .event_dispatchers
      .clone()
      .retain(emitter, |(cb_event_ty, cb)| {
        if *cb_event_ty == event_ty {
          (cb)(event, self)
        } else {
          true
        }
      });
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
      #[expect(clippy::single_match)]
      match effect {
        Effect::Emit {
          emitter,
          event,
          event_ty,
        } => self.apply_emit(&emitter, event_ty, &*event),
        _ => {}
      }
    }
  }

  pub fn focus_handle(&self) -> FocusHandle {
    FocusHandle::new(&self.focus_map)
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

  pub const fn entity_id(&self) -> EntityId {
    self.entity.id()
  }
  pub fn notify(&mut self) {
    self.app.notify(self.entity_id());
  }
  pub fn emit<Event>(&mut self, event: Event)
  where
    Event: 'static,
  {
    self.app.pending_effects.push_back(Effect::Emit {
      emitter: self.entity_id(),
      event: Box::new(event),
      event_ty: TypeId::of::<Event>(),
    });
  }

  pub fn spawn<AsyncFn, R>(&self, f: AsyncFn) -> Task<R>
  where
    E: 'static,
    AsyncFn: 'static + AsyncFnOnce(Entity<E>, &mut AsyncApp) -> R,
    R: 'static,
  {
    let this = self.entity.clone();
    self.app.spawn(async move |cx| f(this, cx).await)
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
