use std::any::TypeId;

#[allow(unused_imports)]
use crate::bundle::{Bundle, Component, Components};
#[cfg(feature = "system")]
use crate::system::state::AliasMap;

/// # 介绍
///
/// 将数据从[Components]转化为[WorldFetch::Item]的格式
///
/// # 原理
///
/// 每种[Bundle]都可能通过[WorldFetch::contain]生成一个[MappingTable]
///
/// 然后根据[MappingTable]生成统一的[WorldFetch::Item]
///
/// # 例子
///
/// [WorldFetch]为[i32]
///
/// | Bundle | Mapping | Item |
/// | :---:  | :---:   | ---  |
/// (usize,i32)| 1     | i32 |
/// (String,i32)|1     | i32 |
/// (i32,Vec<i32>)|0   | i32 |
///
/// 上面三种[Bundle]中都有相同的[Component] : [i32]
///
/// 那么每种[Bundle]根据[WorldFetch]生成一个[MappingTable],
/// 就可以根据[MappingTable]生成统一的Item
#[derive(Debug, Clone)]
pub enum MappingTable {
    Node(Vec<MappingTable>),
    Mapping(usize),
}

impl MappingTable {
    pub fn as_node(&self) -> Option<&Vec<MappingTable>> {
        if let Self::Node(v) = self {
            Some(v)
        } else {
            None
        }
    }

    pub fn as_mapping(&self) -> Option<&usize> {
        if let Self::Mapping(v) = self {
            Some(v)
        } else {
            None
        }
    }
}

/// 从[World]中筛选[Bundle],并且转换[Bundle]
///
/// 并且通过从不同[Bundle]生成不同[MappingTable],
/// 来做到生成同一[WorldFetch::Item]
pub trait WorldFetch {
    /// 转化的目标 通常即就是实现这个特征的类型
    type Item<'a>;

    type Bundle: Bundle;

    /// 从[Componnets],根据[MappingTable]生成[WorldFetch::Item]
    ///
    /// # Safety
    ///
    /// 因为绕开了rust的别名模型,并且进行了一系列类型转换,标记为unsafe
    ///
    /// 安全性实际上已经由[FnSystem]保证
    ///
    /// [FnSystem]: crate
    unsafe fn build<'a>(components: &'a Components, mapping_table: &MappingTable)
        -> Self::Item<'a>;

    /// 通过[Bundle]的信息生成[MappingTable]
    ///
    /// + 返回[Some]说明可以通过[MappingTable]转换[Components]为[WorldFetch::Item]
    /// + 返回[None]代表无法转换
    fn contain(components_ids: &mut Vec<TypeId>) -> Option<MappingTable>;

    /// 计算[WorldFetch]可能导致的别名冲突
    ///
    /// 如果存在别名冲突,带有发生冲突的[WorldFetch]在第一次执行时会发生painc
    #[cfg(feature = "system")]
    fn alias_conflict(alias_map: &mut AliasMap);
}

impl<T: Component> WorldFetch for &T {
    type Item<'a> = &'a T;

    type Bundle = &'static T;

    unsafe fn build<'a>(
        components: &'a Components,
        mapping_table: &MappingTable,
    ) -> Self::Item<'a> {
        components[mapping_table.as_mapping().copied().unwrap()]
            .downcast_ref()
            .unwrap()
    }

    fn contain(components_ids: &mut Vec<TypeId>) -> Option<MappingTable> {
        let mapping = components_ids.binary_search(&TypeId::of::<T>()).ok()?;
        components_ids.remove(mapping);

        Some(MappingTable::Mapping(mapping))
    }

    #[cfg(feature = "system")]
    fn alias_conflict(alias_map: &mut AliasMap) {
        alias_map.insert::<Self, T>(crate::system::state::Alias::Imut)
    }
}

impl<T: Component> WorldFetch for &'_ mut T {
    type Item<'a> = &'a mut T;

    type Bundle = &'static mut T;

    unsafe fn build<'a>(
        components: &'a Components,
        mapping_table: &MappingTable,
    ) -> Self::Item<'a> {
        let imref = components[mapping_table.as_mapping().copied().unwrap()]
            .downcast_ref::<T>()
            .unwrap();
        // 编译器有很努力防止我破坏别名模型
        #[allow(mutable_transmutes)]
        std::mem::transmute(imref)
    }

    fn contain(components_ids: &mut Vec<TypeId>) -> Option<MappingTable> {
        let mapping = components_ids.binary_search(&TypeId::of::<T>()).ok()?;
        components_ids.remove(mapping);
        Some(MappingTable::Mapping(mapping))
    }

    #[cfg(feature = "system")]
    fn alias_conflict(alias_map: &mut AliasMap) {
        alias_map.insert::<Self, T>(crate::system::state::Alias::Mut)
    }
}

#[rustfmt::skip]
mod __impl {
    
    use super::{Components, MappingTable, TypeId, WorldFetch,};
    #[cfg(feature = "system")]
    use super::AliasMap;

    macro_rules! impl_fetch {
        ($($t:ident),*) => {
            impl<$($t:WorldFetch),*> WorldFetch for ($($t,)*){
                type Item<'a> = ($($t::Item<'a>,)*);

                type Bundle = ($($t::Bundle,)*);

                unsafe fn build<'a>(
                    components: &'a Components,
                    mapping_table: &MappingTable,
                ) -> Self::Item<'a> {
                    let mut mappings = mapping_table.as_node().unwrap().into_iter();
                    ($(
                        $t::build(components,mappings.next().unwrap()),
                    )*)
                }

                fn contain(components_ids : &mut Vec<TypeId>) -> Option<MappingTable>{
                    let  mappings = vec![
                        $(
                            $t::contain(components_ids)?,
                        )*
                    ];
                    
                    Some(MappingTable::Node(mappings))
                }

                #[cfg(feature = "system")]
                fn alias_conflict(alias_map: &mut AliasMap) {
                    $($t::alias_conflict(alias_map);)*
                }
            }
        };
    }
    
    // 一次性从(T0)impl到(T0,T1,..,T15)
    proc::all_tuple!(impl_fetch, 16);
}
