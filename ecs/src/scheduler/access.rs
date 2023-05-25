use std::{
    marker::PhantomData,
    ops::{Deref, DerefMut},
};

use crate::{component::entity::Entity, world::chunk::Chunk};

use super::{
    fetch::{MappingTable, WorldFetch},
    query::WorldQuery,
};

/// [WorldFetch]和[WorldQuery]的实际选取
///
/// 通过迭代获取
pub struct Access<'a, F: WorldFetch, Q: WorldQuery> {
    mapping: &'a MappingTable,
    chunk: &'a Chunk,
    chunk_index: usize,
    components_index: usize,
    _f: PhantomData<F>,
    _q: PhantomData<Q>,
}

pub struct Components<I> {
    pub(crate) inner: I,
    pub(crate) entity: Entity,
}

impl<I> Components<I> {
    #[inline]
    pub fn entity(&self) -> Entity {
        self.entity
    }
}

impl<I> Deref for Components<I> {
    type Target = I;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<I> DerefMut for Components<I> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}
