use std::{any::TypeId, collections::BTreeMap};

use super::{
    fetch::{MappingTable, WorldFetch},
    query::WorldQuery,
    Scheduler,
};

/// 映射的缓存
pub struct MappingCache {
    /// * [WorldQuery]是否通过
    pub(crate) query: BTreeMap<TypeId, bool>,
    /// * [WorldFetch]产生的[MappingTable]以及其展开
    pub(crate) fetch: BTreeMap<TypeId, Option<MappingTable>>,
    /// * 当前缓存的[Bundle]的类型
    ///
    /// [Bundle]: crate
    pub(crate) components_ids: &'static [TypeId],
}

impl MappingCache {
    pub fn new(components_ids: &'static [TypeId]) -> Self {
        Self {
            components_ids,
            query: Default::default(),
            fetch: Default::default(),
        }
    }

    pub fn get_query<Q: WorldQuery>(&mut self) -> bool {
        let quert_id = TypeId::of::<Q>();
        if !self.query.contains_key(&quert_id) {
            self.query.insert(quert_id, Q::pass(self.components_ids));
        }
        *self.query.get(&quert_id).unwrap()
    }

    fn fetch<F: WorldFetch>(&mut self) {
        let fetch_id = TypeId::of::<F>();
        if !self.fetch.contains_key(&fetch_id) {
            let mut components = self.components_ids.to_vec();
            let mapping = F::contain(&mut components);
            self.fetch.insert(fetch_id, mapping);
        }
    }

    pub fn get_fetch_mapping<F: WorldFetch>(&mut self) -> Option<&MappingTable> {
        let fetch_id = TypeId::of::<F>();
        self.fetch::<F>();
        match self.fetch.get(&fetch_id)? {
            Some(mapping_table) => Some(mapping_table),
            None => None,
        }
    }
}

/// 每组的调度缓存
pub struct ScheduleCache {
    /// * 所有System的所有Assess的[WorldFetch],[WorldQuery]
    pub(crate) access: Vec<(TypeId, TypeId)>,
    /// * 与[WorldQuery]和[WorldFetch]对应的[MappingTable]生成器/匹配器
    pub(crate) vtables: Vec<(
        // [WorldFetch]
        fn(components: &mut Vec<TypeId>) -> Option<MappingTable>,
        // [WorldQuery]
        fn(components: &[TypeId]) -> bool,
    )>,
    /// * 每个[WorldFetch]和其他[WorldFetch]的冲突情况
    ///
    /// ### 储存规则
    ///
    /// 第一个储存A
    ///
    /// 第二个储存B
    ///
    /// 那么A B就是冲突的
    ///
    /// 依次类推
    ///
    pub(crate) conflict: Vec<(usize, usize)>,
}

impl ScheduleCache {
    pub fn new() -> Self {
        Self {
            access: Default::default(),
            conflict: Default::default(),
            vtables: Default::default(),
        }
    }

    pub fn sync(&mut self, scheduler: &mut Scheduler) {
        let (world, mapping_caches) = (&mut scheduler.world, &mut scheduler.mapping_cache);
        for (bundle_id, meta) in &world.metas {
            // 如果类型还没有缓存  那么就创建一个缓存

            if !mapping_caches.contains_key(&bundle_id) {
                let mapping_cache = MappingCache::new(meta.components);
                mapping_caches.insert(*bundle_id, mapping_cache);
            }
            let mapping_cache = mapping_caches.get_mut(&bundle_id).unwrap();
            // 冲突可能发生
            // 能够生成MappingTable的位置会保存MappingTable的展开
            let mut conflict: Vec<Option<Vec<usize>>> = Vec::with_capacity(self.access.len());

            // 可能类型已经有缓存: 但是没有对access缓存
            // 每一组system的fetch,query
            for (access_id, (fetch, query)) in self.access.iter().enumerate() {
                // 分别给query和fetch插入内容
                let contain = (self.vtables[access_id].1)(meta.components);
                mapping_cache.query.insert(*query, contain);
                // 如果Query已经不通过 那么Fetch也没有计算的必要
                if contain {
                    let mapping_table = (self.vtables[access_id].0)(&mut meta.components.to_vec());
                    let mut expansion = None;
                    // 如果可以,进行展开
                    let mapping_table = mapping_table.and_then(|mapping_table| {
                        expansion = Some(mapping_table.expansion());
                        Some(mapping_table)
                    });

                    conflict.push(expansion);
                    mapping_cache.fetch.insert(*fetch, mapping_table);
                }
            }

            // 统计冲突
            // 至少两个才可以产生冲突
            if conflict.len() >= 2 {
                // 逐个匹配
                for cmp_a in 0..conflict.len() - 1 {
                    for cmp_b in cmp_a..conflict.len() {
                        // 使用模式匹配来高效开Option
                        match (&conflict[cmp_a], &conflict[cmp_b]) {
                            (&Some(ref lhs), &Some(ref rhs)) => {
                                // 遍历L,查看R是否包含
                                for n in lhs {
                                    // 包含意味着发生冲突
                                    if rhs.contains(n) {
                                        self.conflict.push((cmp_a, cmp_b));
                                        break;
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
    }
}
