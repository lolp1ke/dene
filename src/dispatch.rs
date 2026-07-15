// SPDX-License-Identifier: Apache-2.0

use std::{
  any::Any,
  cell::RefCell,
  ops::{Index, IndexMut},
  rc::Rc,
  sync::Arc,
};

use smallvec::SmallVec;

use crate::{Action, ActionRegistry, App, Keybinds, Keystroke, Window};

#[derive(Debug)]
#[derive(Clone, Copy)]
#[derive(PartialEq)]
pub(crate) struct DispatchNodeId(pub(crate) usize);
impl Index<DispatchNodeId> for Vec<DispatchNode> {
  type Output = DispatchNode;
  fn index(&self, index: DispatchNodeId) -> &Self::Output {
    &self[index.0]
  }
}
impl IndexMut<DispatchNodeId> for Vec<DispatchNode> {
  fn index_mut(&mut self, index: DispatchNodeId) -> &mut Self::Output {
    &mut self[index.0]
  }
}

#[derive(Debug)]
pub(crate) struct DispatchTree {
  pub(crate) node_stack: Vec<DispatchNodeId>,
  pub(crate) nodes: Vec<DispatchNode>,
  context_stack: Vec<DispatchContext>,
  actions: Rc<ActionRegistry>,
  keybinds: Rc<RefCell<Keybinds>>,
}
impl DispatchTree {
  pub(crate) fn new(
    actions: Rc<ActionRegistry>,
    keybinds: Rc<RefCell<Keybinds>>,
  ) -> Self {
    Self {
      node_stack: Vec::new(),
      nodes: Vec::new(),
      context_stack: Vec::new(),
      actions,
      keybinds,
    }
  }

  pub(crate) fn clear(&mut self) {
    self.node_stack.clear();
    self.nodes.clear();
    self.context_stack.clear();
  }

  pub(crate) fn dispatch_keystroke(
    &self,
    pending: &[Keystroke],
    keystroke: &Keystroke,
  ) -> DispatchKeystrokeResult {
    let input = pending
      .iter()
      .chain(std::iter::once(keystroke))
      .collect::<SmallVec<[&Keystroke; 2]>>();
    let keybinds = self.keybinds.borrow();
    let (exact, pending) = keybinds.match_input(input.as_slice());

    match exact.first() {
      Some(binding) => {
        let action_name = binding.action.name();
        let action = self.actions.get_by_name(action_name);
        DispatchKeystrokeResult::Match(action)
      }
      None if pending => DispatchKeystrokeResult::Pending,
      None => DispatchKeystrokeResult::Nope,
    }
  }

  pub(crate) fn dispatch_path(
    &self,
    target: DispatchNodeId,
  ) -> SmallVec<[DispatchNodeId; 8]> {
    let mut dispatch_path = SmallVec::new();
    let mut current_node_id = Some(target);
    while let Some(node_id) = current_node_id {
      dispatch_path.push(node_id);
      current_node_id = self.nodes.get(node_id.0).and_then(|node| node.parent);
    }
    dispatch_path.reverse();
    dispatch_path
  }
  pub(crate) fn node(&self, node_id: &DispatchNodeId) -> &DispatchNode {
    &self.nodes[*node_id]
  }

  pub(crate) fn push_node(&mut self) -> DispatchNodeId {
    let parent = self.node_stack.last().copied();
    let node_id = DispatchNodeId(self.nodes.len());
    self.nodes.push(DispatchNode {
      parent,
      ..Default::default()
    });
    self.node_stack.push(node_id);
    node_id
  }
  pub(crate) fn pop_node(&mut self) {
    let node = &self.nodes[self.active_node_id().unwrap()];
    if node.context.is_some() {
      self.context_stack.pop();
    };
    self.node_stack.pop();
  }
  pub(crate) fn set_active_node(&mut self, node_id: DispatchNodeId) {
    let parent = self.nodes[node_id].parent;
    while self.node_stack.last().copied() != parent
      && !self.node_stack.is_empty()
    {
      self.pop_node();
    }

    if self.node_stack.last().copied() == parent {
      self.node_stack.push(node_id);
      let active_node = &self.nodes[node_id];
      if let Some(context) = active_node.context.clone() {
        self.context_stack.push(context);
      };
    } else {
      todo!();
    };
  }

  pub(crate) fn on_key_event(&mut self, listener: KeyListener) {
    self.active_node().key_listeners.push(listener);
  }

  fn active_node_id(&self) -> Option<DispatchNodeId> {
    self.node_stack.last().copied()
  }
  fn active_node(&mut self) -> &mut DispatchNode {
    let idx = self.active_node_id().unwrap();
    &mut self.nodes[idx]
  }
  pub(crate) fn root_node_id(&self) -> DispatchNodeId {
    DispatchNodeId(0)
  }
}

type KeyListener = Rc<dyn Fn(&dyn Any, DispatchPhase, &mut Window, &mut App)>;
#[derive(derive_more::Debug)]
#[derive(Default)]
pub(crate) struct DispatchNode {
  pub(crate) parent: Option<DispatchNodeId>,
  context: Option<DispatchContext>,

  #[debug("key_listeners.len({})", key_listeners.len())]
  pub(crate) key_listeners: Vec<KeyListener>,
}

#[derive(Debug)]
#[derive(Clone)]
pub(crate) struct DispatchContext(Vec<DispatchContextEntry>);

#[derive(Debug)]
#[derive(Clone)]
struct DispatchContextEntry {
  pub(crate) key: Arc<str>,
  pub(crate) value: Option<Arc<str>>,
}

#[derive(Debug)]
pub(crate) enum DispatchPhase {
  Capture,
  Bubble,
}

#[derive(Debug)]
pub(crate) enum DispatchKeystrokeResult {
  Match(Box<dyn Action>),
  Pending,
  Nope,
}
