use std::{
    marker::PhantomData,
    ops::{Deref, DerefMut},
};

use crate::{
    component::entity::Entity,
    world::chunk::{Chunk, ALIVE_FLAG, CHUNK_SIZE},
};

use super::{
    fetch::{MappingTable, WorldFetch},
    query::WorldQuery,
};

/// 对于单一[Bundle]的所有[Chunk]的迭代器
///
/// [Iterator::Item]为[WorldFetch::Item]
///
/// [Bundle]: crate
pub struct BundleIter<'a, F: WorldFetch, Q: WorldQuery> {
    pub(crate) mapping: &'a MappingTable,
    pub(crate) chunks: &'a Vec<Chunk>,
    pub(crate) selected: Vec<usize>,
    pub(crate) index: usize,
    _f: PhantomData<F>,
    _q: PhantomData<Q>,
}

impl<'a, F: WorldFetch, Q: WorldQuery> Iterator for BundleIter<'a, F, Q> {
    type Item = F::Item<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let chunk_id = self.selected.last()?;
        let chunk = &self.chunks[*chunk_id];

        // item_index < CHUNK_SIZE 并且对应的数据有效
        while self.index < CHUNK_SIZE && !chunk.alive[self.index] > ALIVE_FLAG {
            self.index += 1;
        }

        // 如果在这个Chunk的迭代结束 就迭代下一个chunk
        if self.index >= CHUNK_SIZE {
            self.index = 0;
            self.selected.pop();
            return self.next();
        }
        let item = unsafe { F::build(&chunk.storage[self.index], self.mapping) };
        self.index += 1;
        Some(item)
    }
}

impl<'a, F: WorldFetch, Q: WorldQuery> BundleIter<'a, F, Q> {
    pub(crate) fn new(
        mapping: &'a MappingTable,
        chunks: &'a Vec<Chunk>,
        selected: Vec<usize>,
        index: usize,
    ) -> Self {
        Self {
            mapping,
            chunks,
            selected,
            index,
            _f: PhantomData,
            _q: PhantomData,
        }
    }
}

/// 实体组件
///
/// 在[WorldFetch::Item]之外还保留了[Entity]
///
/// 也就可以根据[Entity]做到"删除","返回"[Components]
///
/// [Components]: crate
pub struct EntityBundle<I> {
    pub(crate) inner: I,
    pub(crate) entity: Entity,
}

impl<I> EntityBundle<I> {
    #[inline]
    pub fn entity(&self) -> Entity {
        self.entity
    }
}

impl<I> Deref for EntityBundle<I> {
    type Target = I;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<I> DerefMut for EntityBundle<I> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

/// 对于单一[Bundle]的所有[Chunk]的迭代器
///
/// [Iterator::Item]为[EntityBundle<WorldFetch::Item>]
///
/// 相比于[BundleIter]额外附带了[Iterator::Item]对应的[Entity]
///
/// [Bundle]: crate
pub struct EntityBundleIter<'a, F: WorldFetch, Q: WorldQuery> {
    pub(super) inner: BundleIter<'a, F, Q>,
}

impl<'a, F: WorldFetch, Q: WorldQuery> Iterator for EntityBundleIter<'a, F, Q> {
    type Item = EntityBundle<F::Item<'a>>;

    fn next(&mut self) -> Option<Self::Item> {
        // 实际上的物品
        let inner = self.inner.next()?;
        // 区块的id
        let chunk_id = *self.inner.selected.last().unwrap();
        // 因为每次next之后index都会 加一 所以减一
        let index = self.inner.index - 1;
        // index对应的alive数据,
        let generator = self.inner.chunks[chunk_id].alive[index];
        let entity = Entity { index, generator };

        Some(EntityBundle { inner, entity })
    }
}

/// 对于所有通过[WorldQuery]并且被[WorldFetch]生成[MappingTable]
/// 的[Bundle]的所有[Chunk]的迭代器
///
/// [Iterator::Item]为[WorldFetch::Item]
///
/// [Bundle]: crate
pub struct BundleIters<'a, F: WorldFetch, Q: WorldQuery> {
    pub(crate) iters: Vec<BundleIter<'a, F, Q>>,
}

impl<'a, F: WorldFetch, Q: WorldQuery> Iterator for BundleIters<'a, F, Q> {
    type Item = F::Item<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        let iter = self.iters.last_mut()?;
        let item = iter.next();
        if item.is_none() {
            self.iters.pop();
            return self.next();
        }
        item
    }
}

/// 对于所有通过[WorldQuery]并且被[WorldFetch]生成[MappingTable]
/// 的[Bundle]的所有[Chunk]的迭代器
///
/// [Iterator::Item]为[EntityBundle<WorldFetch::Item>]
/// 相比于[BundleIters]额外附带了[Iterator::Item]对应的[Entity]
///
/// [Bundle]: crate
pub struct EntityBundleIters<'a, F: WorldFetch, Q: WorldQuery> {
    pub(crate) iters: Vec<EntityBundleIter<'a, F, Q>>,
}

impl<'a, F: WorldFetch, Q: WorldQuery> Iterator for EntityBundleIters<'a, F, Q> {
    type Item = EntityBundle<F::Item<'a>>;

    fn next(&mut self) -> Option<Self::Item> {
        let iter = self.iters.last_mut()?;
        let item = iter.next();
        if item.is_none() {
            self.iters.pop();
            return self.next();
        }
        item
    }
}
