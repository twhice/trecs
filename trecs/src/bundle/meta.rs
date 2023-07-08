use std::{
    any::{type_name, TypeId},
    collections::HashMap,
    fmt::Debug,
};

use crate::tools::{MappingTable, WorldFetch, WorldFilter};

use super::{Bundle, Components};

/// 一个[Bundle]的信息
///
/// 一个[Bundle]由若干个[Component]组成
///
/// 比如(123,&&str) 就是一个Bundle
pub struct BundleMeta {
    /// [Bundle]的typeid
    pub bundle_id: TypeId,
    /// [Bundle]的所有Componenets的id
    pub components_ids: &'static [TypeId],
    /// [Bundle]对于每种[WorldFilter]的结果
    ///
    /// 避免每次都重新计算
    pub filter_cache: HashMap<TypeId, bool>,
    /// 每种[WorldFetch]对于此[Bundle]的[MappingTable]
    ///
    /// 避免每次都重新计算
    pub fetch_cache: HashMap<TypeId, MappingTable>,
    /// [World]中所有存放此类[Bundle]的[Chunk]的下标
    pub chunks: Vec<usize>,

    /// 类似[World]的resources_dropers
    ///
    /// 是一个用来还原并drop[Bundle]的函数
    pub droper: Box<dyn Fn(Components)>,

    bundle_info: (&'static str, &'static str),
}

impl BundleMeta {
    pub fn new<B: Bundle>() -> Self {
        let droper = |cs: Components| B::drop(cs);
        Self {
            bundle_id: B::type_id_(),
            components_ids: B::components_ids(),
            filter_cache: Default::default(),
            fetch_cache: Default::default(),
            chunks: vec![],
            bundle_info: (type_name::<B>(), B::type_name()),
            droper: Box::new(droper),
        }
    }

    pub fn filter<F: WorldFilter>(&mut self) -> bool {
        let filter_id = TypeId::of::<F>();

        self.filter_cache
            .entry(filter_id)
            .or_insert_with(|| F::filter(self.components_ids))
            .to_owned()
    }

    pub fn fetch<F: WorldFetch>(&mut self) -> Option<&MappingTable> {
        let fetch_id = F::Bundle::type_id_();
        if let std::collections::hash_map::Entry::Vacant(e) = self.fetch_cache.entry(fetch_id) {
            let mapping_table = F::contain(&mut self.components_ids.to_vec());
            if let Some(mapping_table) = mapping_table {
                e.insert(mapping_table);
            }
        }
        self.fetch_cache.get(&fetch_id)
    }
}

impl Debug for BundleMeta {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BundleMeta")
            .field("bundle_id", &self.bundle_info.0)
            .field("components_ids", &self.bundle_info.1)
            .field("filter_cache", &self.filter_cache)
            .field("fetch_cache", &self.fetch_cache)
            .field("chunks", &self.chunks)
            .finish()
    }
}
