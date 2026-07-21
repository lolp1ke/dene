// SPDX-License-Identifier: Apache-2.0

use std::{
  any::{Any, TypeId},
  marker::PhantomData,
  ops::{Deref, DerefMut},
  sync::{Arc, atomic::AtomicUsize},
};

use parking_lot::RwLock;
use slotmap::{SecondaryMap, SlotMap};

use crate::{
  AnyElement, AnyView, App, AppContext, Context, FocusHandle, Focusable,
  IntoElement, Render,
};

slotmap::new_key_type! {
  pub struct EntityId;
}

#[derive(Debug)]
#[derive(derive_more::Deref, derive_more::DerefMut)]
pub struct Entity<E> {
  #[deref]
  #[deref_mut]
  any: AnyEntity,
  ty: PhantomData<E>,
}
impl<E> Entity<E> {
  pub(crate) const fn new(entity_id: EntityId) -> Self
  where
    E: 'static,
  {
    Self {
      any: AnyEntity::new(entity_id, TypeId::of::<E>()),
      ty: PhantomData,
    }
  }

  pub(crate) fn into_any(self) -> AnyEntity {
    self.any
  }

  pub(crate) fn id(&self) -> EntityId {
    self.any.entity_id
  }

  pub fn read<'a>(&self, cx: &'a App) -> &'a E
  where
    E: 'static,
  {
    cx.entities.read(self)
  }
  pub fn update<C, F, R>(&self, cx: &mut C, f: F) -> R
  where
    C: AppContext,
    F: FnOnce(&mut E, &mut Context<E>) -> R,
    E: 'static,
  {
    cx.update_entity(self, f)
  }
}
impl<E> Clone for Entity<E> {
  fn clone(&self) -> Self {
    Self {
      any: self.any.clone(),
      ty: self.ty,
    }
  }
}
impl<F> Focusable for Entity<F>
where
  F: Focusable,
{
  fn focus_handle(&self, cx: &App) -> FocusHandle {
    self.read(cx).focus_handle(cx)
  }
}
impl<V> IntoElement for Entity<V>
where
  V: Render,
{
  type Element = AnyView;

  fn into_element(self) -> Self::Element {
    self.into()
  }
}

#[derive(Debug)]
#[derive(Clone)]
pub struct AnyEntity {
  entity_id: EntityId,
  ty_id: TypeId,
}
impl AnyEntity {
  const fn new(entity_id: EntityId, ty_id: TypeId) -> Self {
    Self { entity_id, ty_id }
  }
  pub(crate) fn downcast<E>(self) -> Option<Entity<E>>
  where
    E: 'static,
  {
    if TypeId::of::<E>() == self.ty_id {
      return Some(Entity {
        any: self,
        ty: PhantomData,
      });
    }
    None
  }
}

#[derive(Debug)]
#[derive(derive_more::Deref, derive_more::DerefMut)]
pub(crate) struct Slot<E>(pub(crate) Entity<E>);

#[derive(Debug)]
#[derive(Default)]
pub(crate) struct EntityMap {
  entities: SecondaryMap<EntityId, Box<dyn Any>>,
  counts: Arc<RwLock<EntityRefCounts>>,
}
impl EntityMap {
  pub(crate) fn reserve<E>(&self) -> Slot<E>
  where
    E: 'static,
  {
    let entity_id = self.counts.write().counts.insert(AtomicUsize::new(1));
    Slot(Entity::new(entity_id))
  }
  pub(crate) fn read<E>(&self, handle: &Entity<E>) -> &E
  where
    E: 'static,
  {
    self.try_read(handle).unwrap()
  }
  pub(crate) fn try_read<E>(&self, handle: &Entity<E>) -> Option<&E>
  where
    E: 'static,
  {
    self
      .entities
      .get(handle.entity_id)
      .and_then(|entity| entity.downcast_ref::<E>())
  }
  pub(crate) fn insert<E>(&mut self, slot: Slot<E>, entity: E) -> Entity<E>
  where
    E: 'static,
  {
    let handle = slot.0;
    self.entities.insert(handle.entity_id, Box::new(entity));
    handle
  }

  pub(crate) fn lease<E>(&mut self, handle: &Entity<E>) -> Lease<E>
  where
    E: 'static,
  {
    let entity = self
      .entities
      .remove(handle.entity_id)
      .unwrap_or_else(|| panic!("already leased"));
    Lease::new(entity, handle.entity_id)
  }
  pub(crate) fn end_lease<E>(&mut self, lease: Lease<E>)
  where
    E: 'static,
  {
    self.entities.insert(lease.entity_id, lease.entity);
  }
}

#[derive(Debug)]
pub(crate) struct Lease<E> {
  entity: Box<dyn Any>,
  entity_id: EntityId,
  entity_ty: PhantomData<E>,
}
impl<E> Lease<E> {
  const fn new(entity: Box<dyn Any>, entity_id: EntityId) -> Self {
    Self {
      entity,
      entity_id,
      entity_ty: PhantomData,
    }
  }
}
impl<E> Deref for Lease<E>
where
  E: 'static,
{
  type Target = E;
  fn deref(&self) -> &Self::Target {
    self.entity.downcast_ref().unwrap()
  }
}
impl<E> DerefMut for Lease<E>
where
  E: 'static,
{
  fn deref_mut(&mut self) -> &mut Self::Target {
    self.entity.downcast_mut().unwrap()
  }
}

#[derive(Debug)]
#[derive(Default)]
struct EntityRefCounts {
  counts: SlotMap<EntityId, AtomicUsize>,
}
