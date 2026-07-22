// SPDX-License-Identifier: Apache-2.0

use std::{
  any::Any, borrow::Borrow, cell::RefCell, collections::BTreeMap, rc::Rc,
};

use slotmap::SlotMap;

pub trait EventDispatcher<Event: Any> {}

slotmap::new_key_type! {
  pub(crate) struct EventDispatcherId;
}

#[derive(Debug)]
pub(crate) struct EventDispatcherSet<Key, Callback> {
  set: Rc<RefCell<BTreeMap<Key, SlotMap<EventDispatcherId, Callback>>>>,
}
impl<Key, Callback> EventDispatcherSet<Key, Callback> {
  pub(crate) fn insert(
    &mut self,
    key: Key,
    value: Callback,
  ) -> EventDispatcherId
  where
    Key: Ord,
  {
    let mut lock = self.set.borrow_mut();
    lock.entry(key).or_default().insert(value)
  }
  pub(crate) fn retain<Q, F>(&mut self, key: &Q, mut f: F)
  where
    Q: ?Sized + Ord,
    Key: Borrow<Q> + Ord,
    F: FnMut(&mut Callback) -> bool,
  {
    let mut lock = self.set.borrow_mut();
    let Some(callbacks) = lock.get_mut(key) else {
      return;
    };
    callbacks.retain(|_, callback| f(callback));
  }
}
impl<Key, Callback> Default for EventDispatcherSet<Key, Callback> {
  fn default() -> Self {
    Self {
      set: Default::default(),
    }
  }
}
impl<Key, Callback> Clone for EventDispatcherSet<Key, Callback> {
  fn clone(&self) -> Self {
    Self {
      set: self.set.clone(),
    }
  }
}
