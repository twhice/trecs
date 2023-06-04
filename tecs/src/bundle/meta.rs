use std::{
    any::{Any, TypeId},
    collections::HashMap,
};

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
    pub fetch_cache: HashMap<TypeId, Box<dyn Any>>,
    /// [World]中所有存放此类[Bundle]的[Chunk]的下标
    pub chunks: Vec<usize>,
}
