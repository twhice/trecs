use std::{any::TypeId, collections::BTreeMap};

use crate::world::World;

use self::caches::MappingCache;

pub(crate) mod access;
pub mod caches;
pub(crate) mod fetch;
pub mod iter;
pub(crate) mod query;
pub(crate) mod system;

/// 常用的无界生命周期转换
///
/// 封装为一个函数
///
/// 极度unsafe 安全性没有保证
#[inline]
pub unsafe fn transmute_lifetime<'a, 'b, T>(x: &'a T) -> &'b mut T {
    &mut *(x as *const _ as *mut T)
}

/// 引用的类型强制转换
///
/// 极度unsafe
#[inline]
pub unsafe fn transmute_ref<T, Y>(x: &T) -> &mut Y {
    &mut *(x as *const _ as *mut Y)
}

/// 调度器
///
/// 旨在避免冲突(比如别名),优化System运行
pub struct Scheduler<'a> {
    /// 世界的副本
    world: &'a mut World,
    /// Access的缓存
    mapping_cache: BTreeMap<TypeId, MappingCache>,
}

impl<'a> Scheduler<'a> {
    pub fn sync(&mut self) {
        for (ty, meta) in &self.world.metas {
            if !self.mapping_cache.contains_key(&ty) {
                self.mapping_cache
                    .insert(*ty, MappingCache::new(meta.components));
            }
        }
    }
}

/// 调度
///
/// 实现此trait 后 就可以作为System的参数
///
///
/// ### 受到调度的有:
///
/// * Access(从[Chunk]中获取数据)
/// (多个&不交叉)
///
/// * Resources(从[World]访问资源)
/// (多个/不存在)
///
/// * Commands(操纵[World],操纵[Entity])
/// (多个/不存在)
///
/// [Chunk]:crate
/// [Entity]:crate
///
/// ### 作用机制
///
/// 每一次可以并行运行N个系统,为一组
///
/// 一组全部运行结束后运行下一组
///
/// * 唯一 指的是每一组至多有一个
///
/// * 多个 指的是每一组可以没有或者有多个
///
/// * 不交叉 指的是保证没有数据竞争
///
/// Access的操作立刻生效
///
/// Resources内部有锁,操作立刻生效,因此可以共享
///
/// Commands直到函数结束后写入数据(惰性的)
///
pub trait Schedule {
    /// 两个[WorldFetch]选取的类型可能会有交叉
    ///
    /// 并且就此产生别名
    ///
    /// 应该避免
    fn is_conflict<'a>(&self, scheduler: &mut Scheduler<'a>) -> bool;
}
