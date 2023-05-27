use std::marker::PhantomData;

use crate::world::chunk::Chunk;

use super::{
    fetch::{MappingTable, WorldFetch},
    iter::{BundleIter, BundleIters, EntityBundleIter, EntityBundleIters},
    query::WorldQuery,
};

/// [WorldFetch]和[WorldQuery]实际生效的地方
///
/// 可以转换为迭代器,Item为[WorldFetch::Item]
///
/// 通过[Access],可以对[World]中所有满足要求的[Bundle]的所有区块中
/// 所有可用的[Components]以特定的格式进行操作(访问,修改)
///
/// * 满足[WorldQuery]的条件
///
/// * [Components]能够按照一个规则转化为[WorldFetch::Item]
///
/// [World]: crate
/// [Bundle]: crate
pub struct Access<'a, F: WorldFetch, Q: WorldQuery = ()> {
    pub(crate) selected_chunks: Vec<(&'a MappingTable, Vec<usize>)>,
    pub(crate) chunks: &'a Vec<Chunk>,
    pub(crate) _ph: PhantomData<(F, Q)>,
}

impl<'a, F: WorldFetch, Q: WorldQuery> Access<'a, F, Q> {
    pub fn bundle_iter(&'a mut self) -> BundleIters<'a, F, Q> {
        let iters = self
            .selected_chunks
            .iter()
            .map(|(mapping, selected)| BundleIter::new(mapping, self.chunks, selected.clone(), 0))
            .collect::<Vec<_>>();

        BundleIters { iters }
    }

    pub fn entity_bundle_iter(&'a mut self) -> EntityBundleIters<'a, F, Q> {
        let iters = self
            .selected_chunks
            .iter()
            .map(|(mapping, selected)| {
                let iter = BundleIter::new(mapping, self.chunks, selected.clone(), 0);
                EntityBundleIter { inner: iter }
            })
            .collect::<Vec<_>>();

        EntityBundleIters { iters }
    }
}
