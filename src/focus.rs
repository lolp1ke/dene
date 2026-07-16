// SPDX-License-Identifier: Apache-2.0

use std::sync::{
  Arc,
  atomic::{self, AtomicUsize},
};

use parking_lot::RwLock;
use slotmap::SlotMap;

use crate::App;

slotmap::new_key_type! {
  pub(crate) struct FocusId;
}

pub trait Focusable: 'static {
  fn focus_handle(&self, cx: &App) -> FocusHandle;
}

#[derive(Debug)]
pub struct FocusHandle {
  pub(crate) id: FocusId,
  pub(crate) focus_map: FocusMap,
  pub(crate) tab_index: isize,
  pub(crate) tab_stop: bool,
}
impl FocusHandle {
  pub(crate) fn new(focus_map: &FocusMap) -> Self {
    let id = focus_map.write().insert(FocusRef {
      rc: AtomicUsize::new(1),
      tab_index: 0,
      tab_stop: false,
    });
    Self {
      id,
      focus_map: focus_map.clone(),
      tab_index: 0,
      tab_stop: false,
    }
  }
}
impl Clone for FocusHandle {
  fn clone(&self) -> Self {
    self
      .focus_map
      .read()
      .get(self.id)
      .unwrap()
      .rc
      .fetch_add(1, atomic::Ordering::SeqCst);
    Self {
      id: self.id,
      focus_map: self.focus_map.clone(),
      tab_index: self.tab_index,
      tab_stop: self.tab_stop,
    }
  }
}
impl Drop for FocusHandle {
  fn drop(&mut self) {
    self
      .focus_map
      .read()
      .get(self.id)
      .unwrap()
      .rc
      .fetch_sub(1, atomic::Ordering::SeqCst);
  }
}

#[derive(Debug)]
#[derive(Clone)]
#[derive(Default)]
#[derive(derive_more::Deref, derive_more::DerefMut)]
pub(crate) struct FocusMap(Arc<RwLock<SlotMap<FocusId, FocusRef>>>);

#[derive(Debug)]
pub(crate) struct FocusRef {
  pub(crate) rc: AtomicUsize,
  pub(crate) tab_index: isize,
  pub(crate) tab_stop: bool,
}
