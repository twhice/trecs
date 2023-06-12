use std::{
    any::{Any, TypeId},
    collections::HashSet,
    marker::PhantomData,
};

use crate::bundle::BundleMeta;
#[allow(unused_imports)]
use crate::{bundle::Bundle, traits::fetch::WorldFetch};
/// 用来过滤[Bundle]
///
/// 作为[WorldFetch]的附属使用
pub trait WorldFilter: Any {
    /// 传入[Bundle]的components_ids
    ///
    /// + 返回true表示通过
    /// + 返回false表示没
    fn filter(components_ids: &'static [TypeId]) -> bool;

    /// 加速版本,会从缓存读取,否则重新计算
    ///
    /// 主要是为了让嵌套的[WorldFilter]可以更快
    fn filter_by_meta(meta: &mut BundleMeta) -> bool;
}

/// [Bundle]是B的子集时通过
#[derive(Debug, Clone, Copy)]
pub struct All<B: Bundle>(PhantomData<B>);

/// [Bundle]与B有交集时通过
#[derive(Debug, Clone, Copy)]
pub struct AnyOf<B: Bundle>(PhantomData<B>);

/// [Bundle]与B没有交集时通过
#[derive(Debug, Clone, Copy)]
pub struct Not<B: Bundle>(PhantomData<B>);

impl<B: Bundle> WorldFilter for All<B> {
    fn filter(components_ids: &'static [TypeId]) -> bool {
        let set = B::components_ids()
            .iter()
            .copied()
            .fold(HashSet::new(), |mut set, id| {
                set.insert(id);
                set
            });
        !components_ids.iter().any(|id| !set.contains(id))
    }

    fn filter_by_meta(meta: &mut BundleMeta) -> bool {
        meta.filter::<Self>()
    }
}

impl<B: Bundle> WorldFilter for AnyOf<B> {
    fn filter(components_ids: &'static [TypeId]) -> bool {
        let set = B::components_ids()
            .iter()
            .copied()
            .fold(HashSet::new(), |mut set, id| {
                set.insert(id);
                set
            });
        components_ids.iter().any(|id| set.contains(id))
    }

    fn filter_by_meta(meta: &mut BundleMeta) -> bool {
        meta.filter::<Self>()
    }
}

impl<B: Bundle> WorldFilter for Not<B> {
    fn filter(components_ids: &'static [TypeId]) -> bool {
        let set = B::components_ids()
            .iter()
            .copied()
            .fold(HashSet::new(), |mut set, id| {
                set.insert(id);
                set
            });
        !components_ids.iter().any(|id| set.contains(id))
    }

    fn filter_by_meta(meta: &mut BundleMeta) -> bool {
        meta.filter::<Self>()
    }
}

mod __impl {
    use super::{BundleMeta, TypeId, WorldFilter};
    macro_rules! impl_filter {
        ($($t:ident),*) => {
            impl<$($t:WorldFilter),*> WorldFilter for ($($t,)*) {
                fn filter(components_ids : &'static [TypeId]) -> bool{
                    $($t::filter(components_ids))&&*
                }

                fn filter_by_meta(meta: &mut BundleMeta) -> bool {
                   $($t::filter_by_meta(meta))&&*
                }
            }
        };
    }

    proc::all_tuple!(impl_filter, 16);

    impl WorldFilter for () {
        fn filter(_: &'static [TypeId]) -> bool {
            true
        }

        fn filter_by_meta(_meta: &mut BundleMeta) -> bool {
            true
        }
    }
}
