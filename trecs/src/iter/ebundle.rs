use std::ops::{Deref, DerefMut};

use crate::{
    storage::Entity,
    tools::{WorldFetch, WorldFilter},
    world::World,
};

use super::Iter;

/// 带有[Entity]的[Bundle]
///
/// 可以通过[Entity]进行操作
pub struct EBundle<'a, F: WorldFetch> {
    entity: Entity,
    bundle: F::Item<'a>,
}

impl<'a, F: WorldFetch> EBundle<'a, F> {
    pub fn new(entity: Entity, bundle: F::Item<'a>) -> Self {
        Self { entity, bundle }
    }

    pub fn entity(&self) -> Entity {
        self.entity
    }
}

impl<'a, F: WorldFetch> Deref for EBundle<'a, F> {
    type Target = F::Item<'a>;

    fn deref(&self) -> &Self::Target {
        &self.bundle
    }
}

impl<'a, F: WorldFetch> DerefMut for EBundle<'a, F> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.bundle
    }
}

pub struct EIter<'a, F: WorldFetch> {
    inner: Iter<'a, F>,
}

impl<'a, F: WorldFetch> EIter<'a, F> {
    pub fn new<Q: WorldFilter>(world: &mut World) -> EIter<'_, F> {
        EIter {
            inner: Iter::new::<Q>(world),
        }
    }
}

impl<'a, F: WorldFetch> From<Iter<'a, F>> for EIter<'a, F> {
    fn from(value: Iter<'a, F>) -> Self {
        Self { inner: value }
    }
}

impl<'a, F: WorldFetch> Iterator for EIter<'a, F> {
    type Item = EBundle<'a, F>;

    fn next(&mut self) -> Option<Self::Item> {
        let item = self.inner.next()?;
        let iter = &self.inner.iter.as_ref()?.1;
        let entity = iter.chunk.gen_entity(iter.index);
        Some(EBundle::new(entity, item))
    }
}
