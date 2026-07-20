// SPDX-License-Identifier: Apache-2.0

use std::{
  any::{Any, TypeId},
  marker::PhantomData,
  rc::Rc,
};

use smallvec::SmallVec;
use taffy::AvailableSpace;

use crate::{
  Action, AnyView, App, AppContext, DispatchKeystrokeResult, DispatchNodeId,
  DispatchPhase, DispatchTree, Entity, FocusHandle, FocusId, FocusTabStopMap,
  InputHandler, IntoElement, KeyDownEvent, KeyUpEvent, KeyboardEvent,
  Keystroke, LayoutEngine, Modifiers, NoAction, Rect, get_terminal,
};

slotmap::new_key_type! {
  pub struct WindowId;
}

#[derive(Debug)]
pub struct Window {
  pub(crate) focus: Option<FocusId>,
  pub(crate) bounds: Rect,
  pub(crate) dirty: bool,

  pub(crate) root: Option<AnyView>,

  pub(crate) current_frame: Frame,
  pub(crate) next_frame: Frame,

  pub(crate) layout_engine: LayoutEngine,
}
impl Window {
  pub(crate) fn new(opts: WindowOptions, cx: &mut App) -> Self {
    let WindowOptions { bounds, .. } = opts;

    Self {
      focus: None,
      bounds,
      dirty: false,
      root: None,
      current_frame: Frame::new(DispatchTree::new(
        cx.actions.clone(),
        cx.keybinds.clone(),
      )),
      next_frame: Frame::new(DispatchTree::new(
        cx.actions.clone(),
        cx.keybinds.clone(),
      )),
      layout_engine: LayoutEngine::new(),
    }
  }

  pub(crate) fn render(&mut self, cx: &mut App) {
    let Some(root) = self.root.as_ref().cloned() else {
      return;
    };
    self.layout_engine.clear();

    let mut root_element = root.into_any_element();
    let root_node_id = root_element.request_layout(self, cx);

    let available_space = taffy::Size {
      width: AvailableSpace::Definite(self.bounds.width as f32),
      height: AvailableSpace::Definite(self.bounds.height as f32),
    };
    self.layout_engine.compute(root_node_id, available_space);

    root_element.pre_render(self, cx);
    get_terminal().write().clear();
    root_element.render(self, cx);
    get_terminal().write().render();

    std::mem::swap(&mut self.next_frame, &mut self.current_frame);
    self.next_frame.clear();
  }

  pub(crate) fn request_layout(
    &mut self,
    style: taffy::Style,
    children: &[taffy::NodeId],
    cx: &mut App,
  ) -> taffy::NodeId {
    self.layout_engine.request_layout(style, children)
  }
  pub(crate) fn layout_bounds(&mut self, node_id: taffy::NodeId) -> Rect {
    self.layout_engine.layout_bounds(node_id)
  }

  pub(crate) fn dispatch_keyboard_event(
    &mut self,
    event: &dyn Any,
    cx: &mut App,
  ) {
    if self.dirty {
      self.render(cx);
    };

    // let keystroke = event
    //   .downcast_ref::<KeyDownEvent>()
    //   .map(|e| e.keystroke.clone())
    //   .unwrap_or_else(|| {
    //     event
    //       .downcast_ref::<KeyUpEvent>()
    //       .unwrap()
    //       .keystroke
    //       .clone()
    //   });

    let node_id = self.focus_in_current_frame(self.focus);
    let dispatch_path =
      &self.current_frame.dispatch_tree.dispatch_path(node_id);

    if let Some(KeyDownEvent { keystroke, is_held }) =
      event.downcast_ref::<KeyDownEvent>()
    {
      cx.propagate_event = true;
      let key_char = keystroke.key_char.clone();
      let modifiers = keystroke.modifiers;
      let pending = &self.current_frame.pending_keystrokes;
      match self
        .current_frame
        .dispatch_tree
        .dispatch_keystroke(pending, keystroke)
      {
        DispatchKeystrokeResult::Match(action) => {
          if action.partial_eq(&NoAction as &dyn Action) {
            return;
          };
          self.current_frame.pending_keystrokes.clear();
          self.dispatch_action_on_node(node_id, &*action, cx);
        }
        DispatchKeystrokeResult::Pending => {
          self
            .current_frame
            .pending_keystrokes
            .push(keystroke.clone());
        }
        DispatchKeystrokeResult::Nope => {
          self.current_frame.pending_keystrokes.clear();
        }
      };

      let mut input_handlers =
        std::mem::take(&mut self.current_frame.input_handlers);
      for input_handler in input_handlers.iter_mut() {
        if !modifiers.intersects(Modifiers::CONTROL)
          && let Some(chars) = key_char.as_deref()
        {
          input_handler.insert_str(None, chars, self, cx);
        };
      }
      self.current_frame.input_handlers.extend(input_handlers);
    };

    self.dispatch_key_down_up_event(event, dispatch_path, cx);
  }

  fn dispatch_key_down_up_event(
    &mut self,
    event: &dyn Any,
    dispatch_path: &[DispatchNodeId],
    cx: &mut App,
  ) {
    for node_id in dispatch_path.iter() {
      let node = self.current_frame.dispatch_tree.node(node_id);

      for listener in node.key_listeners.clone().into_iter() {
        (listener)(event, DispatchPhase::Capture, self, cx);
      }
    }

    for node_id in dispatch_path.iter().rev() {
      let node = self.current_frame.dispatch_tree.node(node_id);

      for listener in node.key_listeners.clone().into_iter() {
        (listener)(event, DispatchPhase::Bubble, self, cx);
      }
    }
  }
  pub(crate) fn dispatch_action_on_node(
    &mut self,
    node_id: DispatchNodeId,
    action: &dyn Action,
    cx: &mut App,
  ) {
    let dispatch_path = self.current_frame.dispatch_tree.dispatch_path(node_id);

    cx.propagate_event = true;
    if let Some(global_action_listeners) = cx
      .global_action_listeners
      .remove(&action.as_any().type_id())
    {
      for listener in global_action_listeners.iter() {
        (listener)(action, DispatchPhase::Capture, cx);
        if !cx.propagate_event {
          break;
        };
      }

      cx.global_action_listeners
        .insert(action.as_any().type_id(), global_action_listeners);
    }

    if !cx.propagate_event {
      return;
    };

    for node_id in dispatch_path.iter() {
      let node = self.current_frame.dispatch_tree.node(node_id);

      for (action_ty_id, listener) in node.action_listeners.clone().into_iter()
      {
        let any_action = action.as_any();
        if action_ty_id == any_action.type_id() {
          (listener)(any_action, DispatchPhase::Capture, self, cx);

          if !cx.propagate_event {
            return;
          };
        };
      }
    }

    for node_id in dispatch_path.iter().rev() {
      let node = self.current_frame.dispatch_tree.node(node_id);

      for (action_ty_id, listener) in node.action_listeners.clone().into_iter()
      {
        let any_action = action.as_any();
        if action_ty_id == any_action.type_id() {
          cx.propagate_event = false;
          (listener)(any_action, DispatchPhase::Bubble, self, cx);
          if !cx.propagate_event {
            return;
          };
        };
      }
    }

    if let Some(global_action_listeners) = cx
      .global_action_listeners
      .remove(&action.as_any().type_id())
    {
      for listener in global_action_listeners.iter().rev() {
        cx.propagate_event = false;
        (listener)(action, DispatchPhase::Bubble, cx);
        if !cx.propagate_event {
          break;
        };
      }

      cx.global_action_listeners
        .insert(action.as_any().type_id(), global_action_listeners);
    }
  }

  pub(crate) fn on_key_event<F, KeyEvent>(&mut self, listener: F)
  where
    F: 'static + Fn(&KeyEvent, DispatchPhase, &mut Self, &mut App),
    KeyEvent: KeyboardEvent,
  {
    self.next_frame.dispatch_tree.on_key_event(Rc::new(
      move |event, phase, window, cx| {
        if let Some(event) = event.downcast_ref::<KeyEvent>() {
          (listener)(event, phase, window, cx);
        };
      },
    ));
  }
  pub(crate) fn on_action<F>(&mut self, action_ty_id: TypeId, listener: F)
  where
    F: 'static + Fn(&dyn Any, DispatchPhase, &mut Window, &mut App),
  {
    self
      .next_frame
      .dispatch_tree
      .on_action(action_ty_id, Rc::new(listener));
  }

  fn focus_in_current_frame(
    &self,
    focus_id: Option<FocusId>,
  ) -> DispatchNodeId {
    focus_id
      .and_then(|focus_id| {
        self.current_frame.dispatch_tree.focusable_node_id(focus_id)
      })
      .unwrap_or_else(|| self.current_frame.dispatch_tree.root_node_id())
  }
  pub(crate) fn set_focus_handle(&mut self, focus_handle: &FocusHandle) {
    if self.focus.is_none() {
      self.focus = Some(focus_handle.id);
    };

    if self.focus == Some(focus_handle.id) {
      self.next_frame.focus = Some(focus_handle.id);
    };
    self.next_frame.dispatch_tree.set_focus_id(focus_handle.id);
  }
  fn focus(&mut self, focus_handle: &FocusHandle) {
    if self.focus == Some(focus_handle.id) {
      return;
    };
    self.focus = Some(focus_handle.id);
    self.dirty = true;
  }
  pub(crate) fn focus_next(&mut self) {
    if let Some(handle) =
      self.current_frame.tab_stop_map.next(self.focus.as_ref())
    {
      self.focus(&handle);
    };
  }
  pub(crate) fn focus_prev(&mut self) {
    if let Some(handle) =
      self.current_frame.tab_stop_map.prev(self.focus.as_ref())
    {
      self.focus(&handle);
    }
  }

  pub fn handle_input<H>(
    &mut self,
    focus_handle: &FocusHandle,
    input_handler: H,
  ) where
    H: InputHandler,
  {
    if focus_handle.is_focused(self) {
      self.next_frame.input_handlers.clear();
      self.next_frame.input_handlers.push(Box::new(input_handler));
    };
  }

  pub(crate) fn with_tab_group<F, R>(
    &mut self,
    tab_index: Option<isize>,
    f: F,
  ) -> R
  where
    F: FnOnce(&mut Self) -> R,
  {
    if let Some(tab_index) = tab_index {
      self.next_frame.tab_stop_map.start_group(tab_index);
      let result = f(self);
      self.next_frame.tab_stop_map.end_group();
      result
    } else {
      f(self)
    }
  }

  pub(crate) fn listener<E, F, AnyEvent>(
    &self,
    view: &Entity<E>,
    f: F,
  ) -> impl 'static + Fn(&AnyEvent, &mut Self, &mut App)
  where
    E: 'static,
    F: 'static + Fn(&mut E, &AnyEvent, &mut Self, &mut App),
  {
    let view = view.clone();
    move |e, window, cx| view.update(cx, |view, cx| f(view, e, window, cx))
  }
}

#[derive(derive_more::Debug)]
pub(crate) struct Frame {
  focus: Option<FocusId>,
  pub(crate) dispatch_tree: DispatchTree,
  pub(crate) pending_keystrokes: SmallVec<[Keystroke; 2]>,
  #[debug(skip)]
  pub(crate) input_handlers: Vec<Box<dyn InputHandler>>,
  pub(crate) tab_stop_map: FocusTabStopMap,
}
impl Frame {
  pub(crate) fn new(dispatch_tree: DispatchTree) -> Self {
    Self {
      focus: None,
      dispatch_tree,
      pending_keystrokes: Default::default(),
      input_handlers: Default::default(),
      tab_stop_map: Default::default(),
    }
  }

  pub(crate) fn clear(&mut self) {
    self.dispatch_tree.clear();
    self.input_handlers.clear();
    self.tab_stop_map.clear();
    // self.focus = None;
  }
}

#[derive(Debug)]
#[derive(Clone, Copy)]
#[derive(derive_more::Deref, derive_more::DerefMut)]
pub struct WindowHandle<W> {
  #[deref]
  #[deref_mut]
  any: AnyWindowHandle,
  _marker: PhantomData<W>,
}
impl<W> WindowHandle<W> {
  pub(crate) fn new(window_id: WindowId) -> Self
  where
    W: 'static,
  {
    Self {
      any: AnyWindowHandle::new::<W>(window_id),
      _marker: PhantomData,
    }
  }
}

#[derive(Debug)]
#[derive(Clone, Copy)]
pub struct AnyWindowHandle {
  pub(crate) window_id: WindowId,
  ty_id: TypeId,
}
impl AnyWindowHandle {
  fn new<W>(window_id: WindowId) -> Self
  where
    W: 'static,
  {
    Self {
      window_id,
      ty_id: TypeId::of::<W>(),
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

#[derive(Debug)]
pub struct WindowOptions {
  pub bounds: Rect,
}
impl Default for WindowOptions {
  fn default() -> Self {
    let (width, height) = crate::Terminal::size();

    Self {
      bounds: Rect {
        x: 0,
        y: 0,
        width,
        height,
      },
    }
  }
}
