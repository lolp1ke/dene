// SPDX-License-Identifier: Apache-2.0

use std::{
  any::{Any, TypeId},
  marker::PhantomData,
  rc::Rc,
};

use ratatui::layout::Rect;
use rustc_hash::FxHashMap;
use slotmap::new_key_type;

use crate::{
  Action, AnyView, App, AppContext, FocusNext, FocusPrev, Keystroke,
  LayoutEngine, PanelId, PanelNode, clamp_area, draw,
};

type ActionListener = Rc<dyn Fn(&dyn Action, &mut Window, &mut App)>;

#[derive(derive_more::Debug)]
pub struct Window {
  handle: AnyWindowHandle,

  pub root: Option<PanelNode>,
  pub active_panel: Option<PanelId>,
  next_pane_id: u32,
  pub(crate) bounds: Rect,

  #[debug(skip)]
  action_listeners: FxHashMap<TypeId, Vec<ActionListener>>,

  layout_engine: LayoutEngine,
}
impl Window {
  pub(crate) fn new(handle: AnyWindowHandle, config: WindowConfig) -> Self {
    let WindowConfig { area, .. } = config;

    let mut action_listeners: FxHashMap<TypeId, Vec<ActionListener>> =
      FxHashMap::default();
    action_listeners
      .entry(FocusNext.type_id())
      .or_default()
      .push(Rc::new(move |_, window, _| {
        window.focus(1);
        tracing::debug!("{:?}", window.active_panel);
      }));
    action_listeners
      .entry(FocusPrev.type_id())
      .or_default()
      .push(Rc::new(move |_, window, _| {
        window.focus(-1);
      }));

    Self {
      handle,
      root: None,
      active_panel: Some(PanelId(0)),
      next_pane_id: 1,

      bounds: area,
      action_listeners,
      layout_engine: LayoutEngine::default(),
    }
  }

  pub fn next_pane_id(&mut self) -> PanelId {
    let id = PanelId(self.next_pane_id);
    self.next_pane_id += 1;
    id
  }

  pub(crate) fn render(&mut self, cx: &mut App) {
    let root_id = self.layout_engine.build(self.root.as_ref().unwrap());
    self.layout_engine.compute(
      root_id,
      self.bounds.width as f32,
      self.bounds.height as f32,
    );

    let views = {
      let mut pairs = Vec::new();
      self.root.as_ref().unwrap().visit_leaves(
        root_id,
        &self.layout_engine,
        self.bounds.x as f32,
        self.bounds.y as f32,
        &mut |pane, rect| pairs.push((pane.view.clone(), rect)),
      );
      pairs
    };

    draw(move |frame| {
      for (view, area) in views.into_iter() {
        let area = clamp_area(area, self.bounds);
        if area.width == 0 || area.height == 0 {
          continue;
        };
        (view.render)(&view, frame, area, self, cx);
      }
    });
  }

  pub(crate) fn dispatch_action(&mut self, action: &dyn Action, cx: &mut App) {
    let action_ty = action.as_any().type_id();
    if let Some(global_listeners) =
      cx.global_action_listeners.remove(&action_ty)
    {
      for listener in global_listeners.iter() {
        (listener)(action, cx);
      }

      cx.global_action_listeners
        .insert(action_ty, global_listeners);
    };

    if let Some(listeners) = self.action_listeners.remove(&action_ty) {
      for listener in listeners.iter() {
        (listener)(action, self, cx);
      }

      self.action_listeners.insert(action_ty, listeners);
    };
  }
  pub(crate) fn dispatch_keystroke(
    &mut self,
    keystroke: Keystroke,
    cx: &mut App,
  ) {
    if let Some(node) = self.root.as_ref()
      && let Some(active_pane) = self.active_panel
      && let Some(pane) = node.find(active_pane)
    {
      let view = pane.view.clone();
      (view.on_keystroke)(&view, keystroke, self, cx);
    };
  }

  pub fn on_action<F, A>(&mut self, f: F)
  where
    F: 'static + Fn(&A, &mut Self, &mut App),
    A: Action,
  {
    self
      .action_listeners
      .entry(TypeId::of::<A>())
      .or_default()
      .push(Rc::new(move |action, window, cx| {
        let action = action.as_any().downcast_ref().expect("wrong action");
        f(action, window, cx);
      }));
  }

  pub(crate) fn focus(&mut self, idx: i32) {
    let Some(root) = self.root.as_ref() else {
      return;
    };
    let order = root.tab_order();
    if order.is_empty() {
      return;
    };
    tracing::debug!("{:?}", order);
    tracing::debug!("{:?}", self.active_panel);
    tracing::debug!("idx: {}", idx);

    let current_idx = self
      .active_panel
      .and_then(|id| order.iter().position(|p| *p == id))
      .unwrap_or(0);
    let focus_idx =
      ((current_idx as i32) + idx).rem_euclid(order.len() as i32) as usize;
    tracing::debug!("cur_idx: {}", current_idx);
    tracing::debug!("focus_idx: {}", focus_idx);
    self.active_panel = Some(order[focus_idx]);
  }
}

#[derive(Debug)]
pub struct WindowConfig {
  pub area: Rect,
}
impl Default for WindowConfig {
  fn default() -> Self {
    let (width, height) = ratatui::crossterm::terminal::size().unwrap();

    Self {
      area: Rect {
        x: 0,
        y: 0,
        width,
        height,
      },
    }
  }
}

new_key_type! {
  pub struct WindowId;
}
#[derive(Debug)]
#[derive(Clone, Copy)]
pub struct AnyWindowHandle {
  pub(crate) window_id: WindowId,
  ty: TypeId,
}
impl AnyWindowHandle {
  fn new<W>(window_id: WindowId) -> Self
  where
    W: 'static,
  {
    Self {
      window_id,
      ty: TypeId::of::<W>(),
    }
  }

  pub(crate) fn update<C, F, R>(self, cx: &mut C, f: F) -> anyhow::Result<R>
  where
    C: AppContext,
    F: FnOnce(AnyView, &mut Window, &mut App) -> R,
  {
    cx.update_window(self, f)
  }
}
impl<W> From<WindowHandle<W>> for AnyWindowHandle {
  fn from(value: WindowHandle<W>) -> Self {
    value.handle
  }
}

#[derive(Debug)]
#[derive(derive_more::Deref, derive_more::DerefMut)]
pub struct WindowHandle<W> {
  #[deref]
  #[deref_mut]
  handle: AnyWindowHandle,
  _marker: PhantomData<W>,
}
impl<W> WindowHandle<W> {
  pub fn new(window_id: WindowId) -> Self
  where
    W: 'static,
  {
    Self {
      handle: AnyWindowHandle::new::<W>(window_id),
      _marker: PhantomData,
    }
  }
}
impl<W> Copy for WindowHandle<W> {}
impl<W> Clone for WindowHandle<W> {
  fn clone(&self) -> Self {
    *self
  }
}
