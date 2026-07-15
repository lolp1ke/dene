// SPDX-License-Identifier: Apache-2.0

use std::{any::TypeId, marker::PhantomData};

use crate::{AnyView, App, FocusId, LayoutEngine, Rect};

slotmap::new_key_type! {
  pub struct WindowId;
}

#[derive(Debug)]
pub struct Window {
  pub(crate) focus: Option<FocusId>,
  pub(crate) bounds: Rect,
  pub(crate) dirty: bool,

  pub(crate) root: Option<AnyView>,

  pub(crate) layout_engine: LayoutEngine,
}
impl Window {
  pub(crate) fn new(opts: WindowOptions) -> Self {
    let WindowOptions { bounds, .. } = opts;

    Self {
      focus: None,
      bounds,
      dirty: false,
      root: None,
      layout_engine: LayoutEngine::new(),
    }
  }

  pub(crate) fn request_layout(
    &mut self,
    style: taffy::Style,
    children: &[taffy::NodeId],
    cx: &mut App,
  ) -> taffy::NodeId {
    self.layout_engine.request_layout(style, children)
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
  window_id: WindowId,
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
}

#[derive(Debug)]
#[derive(Default)]
pub struct WindowOptions {
  bounds: Rect,
}
