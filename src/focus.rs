// SPDX-License-Identifier: Apache-2.0

use std::{
  collections::BTreeSet,
  sync::{
    Arc,
    atomic::{self, AtomicUsize},
  },
};

use parking_lot::RwLock;
use rustc_hash::FxHashMap;
use slotmap::SlotMap;
use smallvec::SmallVec;

use crate::App;

slotmap::new_key_type! {
  pub(crate) struct FocusId;
}

pub trait Focusable: 'static {
  fn focus_handle(&self, cx: &App) -> FocusHandle;
}

#[derive(derive_more::Debug)]
pub struct FocusHandle {
  pub(crate) id: FocusId,
  #[debug(skip)]
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

  pub(crate) fn tab_index(&mut self, tab_index: isize) {
    if let Some(focus_ref) = self.focus_map.write().get_mut(self.id) {
      focus_ref.tab_index = tab_index;
    };
    self.tab_index = tab_index;
  }
  pub(crate) fn tab_stop(&mut self, tab_stop: bool) {
    if let Some(focus_ref) = self.focus_map.write().get_mut(self.id) {
      focus_ref.tab_stop = tab_stop;
    }
    self.tab_stop = tab_stop;
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

#[derive(Debug)]
#[derive(Default)]
pub(crate) struct FocusTabStopMap {
  current_path: TabStopPath,
  insertion_history: Vec<TabStopOperation>,
  by_id: FxHashMap<FocusId, TabStopNode>,
  order: BTreeSet<TabStopNode>,
}
impl FocusTabStopMap {
  pub(crate) fn start_group(&mut self, tab_index: isize) {
    self
      .insertion_history
      .push(TabStopOperation::TabGroup(tab_index));
    self.current_path.0.push(tab_index);
  }
  pub(crate) fn end_group(&mut self) {
    self.insertion_history.push(TabStopOperation::TabGroupEnd);
    self.current_path.0.pop();
  }
  pub(crate) fn insert(&mut self, focus_handle: &FocusHandle) {
    self
      .insertion_history
      .push(TabStopOperation::Handle(focus_handle.clone()));
    let mut path = self.current_path.clone();
    path.0.push(focus_handle.tab_index);
    let order = TabStopNode {
      path,
      node_idx: self.insertion_history.len() - 1,
      tab_stop: focus_handle.tab_stop,
    };
    self.by_id.insert(focus_handle.id, order.clone());
    self.order.insert(order);
  }
  pub(crate) fn clear(&mut self) {
    self.current_path.0.clear();
    self.insertion_history.clear();
    self.by_id.clear();
    self.order.clear();
  }

  pub(crate) fn next(
    &self,
    focused_id: Option<&FocusId>,
  ) -> Option<FocusHandle> {
    let Some(focused_id) = focused_id else {
      let first = self.order.first()?;
      if first.tab_stop {
        return self.focus_handle_for_order(first);
      } else {
        return self
          ._next(first)
          .and_then(|node| self.focus_handle_for_order(node));
      };
    };

    let Some(node) = self.tab_node_for_focus_id(focused_id) else {
      return self.next(None);
    };

    if let Some(item) = self._next(node) {
      self.focus_handle_for_order(item)
    } else {
      self.next(None)
    }
  }
  pub(crate) fn prev(
    &self,
    focused_id: Option<&FocusId>,
  ) -> Option<FocusHandle> {
    let Some(focused_id) = focused_id else {
      let last = self.order.last()?;
      if last.tab_stop {
        return self.focus_handle_for_order(last);
      } else {
        return self
          ._prev(last)
          .and_then(|node| self.focus_handle_for_order(node));
      };
    };

    let Some(node) = self.tab_node_for_focus_id(focused_id) else {
      return self.prev(None);
    };

    if let Some(item) = self._prev(node) {
      self.focus_handle_for_order(item)
    } else {
      self.prev(None)
    }
  }

  fn _next(&self, node: &TabStopNode) -> Option<&TabStopNode> {
    self.order.range(node..).skip(1).find(|node| node.tab_stop)
  }
  fn _prev(&self, node: &TabStopNode) -> Option<&TabStopNode> {
    self.order.range(..node).rev().find(|node| node.tab_stop)
  }

  fn tab_node_for_focus_id(
    &self,
    focused_id: &FocusId,
  ) -> Option<&TabStopNode> {
    self.by_id.get(focused_id)
  }
  fn focus_handle_for_order(&self, order: &TabStopNode) -> Option<FocusHandle> {
    self.insertion_history[order.node_idx]
      .focus_handle()
      .clone()
  }
}

#[derive(Debug)]
enum TabStopOperation {
  Handle(FocusHandle),
  TabGroup(isize),
  TabGroupEnd,
}
impl TabStopOperation {
  fn focus_handle(&self) -> Option<FocusHandle> {
    match self {
      Self::Handle(handle) => Some(handle.clone()),
      _ => None,
    }
  }
}

#[derive(Debug)]
#[derive(Clone)]
#[derive(PartialEq, Eq, PartialOrd, Ord)]
#[derive(Default)]
struct TabStopPath(SmallVec<[isize; 8]>);

#[derive(Debug)]
#[derive(Clone)]
#[derive(PartialEq, Eq)]
struct TabStopNode {
  path: TabStopPath,
  node_idx: usize,
  tab_stop: bool,
}
impl PartialOrd for TabStopNode {
  fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
    Some(self.cmp(other))
  }
}
impl Ord for TabStopNode {
  fn cmp(&self, other: &Self) -> std::cmp::Ordering {
    self
      .path
      .cmp(&other.path)
      .then(self.node_idx.cmp(&other.node_idx))
  }
}
