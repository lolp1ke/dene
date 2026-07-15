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
  DispatchPhase, DispatchTree, Element, FocusId, IntoElement, KeyDownEvent,
  KeyUpEvent, KeyboardEvent, Keystroke, LayoutEngine, NoAction, Rect, window,
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
    root_element.render(self, cx);

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

    let keystroke = event
      .downcast_ref::<KeyDownEvent>()
      .map(|e| e.keystroke.clone())
      .unwrap_or_else(|| {
        event
          .downcast_ref::<KeyUpEvent>()
          .unwrap()
          .keystroke
          .clone()
      });

    let pending = &self.current_frame.pending_keystrokes;
    match self
      .current_frame
      .dispatch_tree
      .dispatch_keystroke(pending, &keystroke)
    {
      DispatchKeystrokeResult::Match(action) => {
        if action.partial_eq(&NoAction as &dyn Action) {
          return;
        };
        self.current_frame.pending_keystrokes.clear();
        cx.dispatch_global_action(&*action);
      }
      DispatchKeystrokeResult::Pending => {
        self.current_frame.pending_keystrokes.push(keystroke);
      }
      DispatchKeystrokeResult::Nope => {
        self.current_frame.pending_keystrokes.clear();
      }
    };

    // TODO: get focused node's id or else fallback to root
    let node_id =
      DispatchNodeId(self.current_frame.dispatch_tree.nodes.len() - 1);
    let dispatch_path =
      &self.current_frame.dispatch_tree.dispatch_path(node_id);

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
}

#[derive(Debug)]
pub(crate) struct Frame {
  focus: Option<FocusId>,
  pub(crate) dispatch_tree: DispatchTree,
  pub(crate) pending_keystrokes: SmallVec<[Keystroke; 2]>,
}
impl Frame {
  pub(crate) fn new(dispatch_tree: DispatchTree) -> Self {
    Self {
      focus: None,
      dispatch_tree,
      pending_keystrokes: Default::default(),
    }
  }

  pub(crate) fn clear(&mut self) {
    self.dispatch_tree.clear();
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
#[derive(Default)]
pub struct WindowOptions {
  bounds: Rect,
}
