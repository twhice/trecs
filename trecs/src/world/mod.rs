use std::{
    any::{Any, TypeId},
    cell::UnsafeCell,
    collections::HashMap,
};

mod commands;
mod query;
mod resources;

pub use self::{
    commands::Commands,
    query::Query,
    resources::{Res, Resources},
};

use crate::{
    bundle::{Bundle, BundleMeta},
    storage::{Chunk, Entity, CHUNK_SIZE},
    tools::{Command, ResManager},
};

/// 这里的[Any]是没有虚表的！！！
/// 不可以进行downcast等操作！！！
/// 否则会导致段错误！！！
type AnRes = UnsafeCell<Option<Box<dyn Any>>>;

type Droper = Option<Box<dyn FnOnce(&mut AnRes)>>;

#[cfg(feature = "system")]
use crate::system::{InnerSystem, System};

pub struct World {
    pub(crate) chunks: Vec<Chunk>,
    pub(crate) metas: HashMap<TypeId, BundleMeta>,
    #[cfg(feature = "system")]
    pub(crate) startup_systems: Vec<System>,
    #[cfg(feature = "system")]
    pub(crate) systems: Vec<System>,
    pub(crate) resources: HashMap<TypeId, AnRes>,
    /// 因为运行时反射 资源在最后都以[Box<dyn Any>]的状态[Drop]
    /// 而不是调用自身的[Drop::drop]和方法
    ///
    /// 所以在创建每一个资源时都记录下一个函数用来Drop
    pub(crate) resources_dropers: HashMap<TypeId, Droper>,
}

impl World {
    pub fn new() -> Self {
        Self {
            chunks: vec![],
            metas: Default::default(),
            #[cfg(feature = "system")]
            startup_systems: vec![],
            #[cfg(feature = "system")]
            systems: vec![],
            resources: Default::default(),
            resources_dropers: Default::default(),
        }
    }

    /// 创建一个新的区块,并且返回它的可变引用
    ///
    /// 防止诸如"meta和实际不一致","chunk.index不正确"等错位问题
    pub(crate) fn new_chunk<B: Bundle>(&mut self) -> &mut Chunk {
        self.metas
            .get_mut(&B::type_id_())
            .unwrap()
            .chunks
            .push(self.chunks.len());
        self.chunks.push(Chunk::new::<B>(self.chunks.len()));
        self.chunks.last_mut().unwrap()
    }
}

#[cfg(feature = "system")]
impl World {
    #[cfg(not(feature = "async"))]
    pub fn exec<M, S: InnerSystem<M>>(&self, mut s: S) {
        s.run_once(s.build_args(self));
    }

    #[cfg(feature = "async")]
    pub async fn exec<M, S: InnerSystem<M>>(&self, mut s: S) {
        s.run_once(s.build_args(self)).await;
    }

    /// 添加一个[System]
    ///
    /// 每次循环都会执行
    pub fn add_system<M, S: InnerSystem<M>>(&mut self, system: S) -> &mut Self {
        self.systems.push(System::new(system));
        self
    }

    /// 添加一个[System]
    ///
    /// 只会在刚开始循环时执行一次
    pub fn add_startup_system<M, S: InnerSystem<M>>(&mut self, system: S) -> &mut Self {
        self.startup_systems.push(System::new(system));
        self
    }

    /// 进入一个死循环,直到线程终结
    ///
    /// 在执行一次所有被添加进startup_systems的[System]后
    ///
    /// 会进入循环,每次循环执行systems里的所有[System]
    #[cfg(not(feature = "async"))]
    pub fn run(&mut self) {
        self.run_until(|| false)
    }
    #[cfg(feature = "async")]
    pub async fn run(&mut self) {
        self.run_until(|| false).await;
    }

    #[cfg(not(feature = "async"))]
    pub fn run_until<F>(&mut self, mut until: F)
    where
        F: FnMut() -> bool,
    {
        loop {
            if until() {
                return;
            }

            self.startup();
            self.run_once();
        }
    }

    #[cfg(feature = "async")]
    pub async fn run_until<F>(&mut self, mut until: F)
    where
        F: FnMut() -> bool,
    {
        loop {
            if until() {
                return;
            }

            self.startup().await;
            self.run_once().await;
        }
    }

    #[cfg(not(feature = "async"))]
    pub fn startup(&mut self) -> &mut Self {
        while let Some(mut stsys) = self.startup_systems.pop() {
            stsys.run_once(self);
        }
        self
    }

    #[cfg(feature = "async")]
    pub async fn startup(&mut self) -> &mut Self {
        while let Some(mut stsys) = self.startup_systems.pop() {
            stsys.run_once(self).await;
        }
        self
    }

    /// 执行一次所有system
    #[cfg(not(feature = "async"))]
    pub fn run_once(&mut self) {
        let this = unsafe {
            // stable没下面的"cast_ref_to_mut" 所以需要下面的allow
            #[allow(unknown_lints)]
            // nightly版本会deny 所以这需要allow
            #[allow(clippy::cast_ref_to_mut)]
            &mut *(self as *const _ as *mut World)
        };
        for sys in &mut self.systems {
            sys.run_once(this);
        }
    }
    #[cfg(feature = "async")]
    pub async fn run_once(&mut self) {
        let this = unsafe {
            // stable没下面的"cast_ref_to_mut" 所以需要下面的allow
            #[allow(unknown_lints)]
            // nightly版本会deny 所以这需要allow
            #[allow(clippy::cast_ref_to_mut)]
            &mut *(self as *const _ as *mut World)
        };
        for sys in &mut self.systems {
            sys.run_once(this).await;
        }
    }
}

impl Default for World {
    fn default() -> Self {
        Self::new()
    }
}

/// 反转[Result<T,E>]的T和E
///
/// 为了蹭语法糖写的垃圾
fn rev_result<T, E>(result: Result<T, E>) -> Result<E, T> {
    match result {
        Ok(o) => Err(o),
        Err(e) => Ok(e),
    }
}

impl Command for World {
    fn register<B: crate::bundle::Bundle>(&mut self) {
        let bundle_id = B::type_id_();
        self.metas
            .entry(bundle_id)
            .or_insert_with(|| BundleMeta::new::<B>());
    }

    fn spawn<B: crate::bundle::Bundle>(&mut self, b: B) -> crate::storage::Entity {
        self.register::<B>();
        let bundle_id = B::type_id_();
        let mut bundle = Some(b);

        let meta = self.metas.get_mut(&bundle_id).unwrap();

        meta.chunks
            .iter()
            .try_fold((), |_, &cid| {
                // Result<(),Entity>
                bundle = Some(rev_result(self.chunks[cid].insert(bundle.take().unwrap()))?);
                Ok(())
            })
            .err()
            .unwrap_or_else(|| self.new_chunk::<B>().insert(bundle?).ok())
            .unwrap()
    }

    fn spawn_many<B: crate::bundle::Bundle, I: IntoIterator<Item = B>>(
        &mut self,
        i: I,
    ) -> Vec<Entity> {
        // 注册&&准备meta
        self.register::<B>();
        // let meta = self.metas.get_mut(&B::type_id_()).unwrap();

        // 准备迭代器和返回
        let mut i = i.into_iter();
        let mut chuns_iter = self
            .metas
            .get_mut(&B::type_id_())
            .unwrap()
            .chunks
            .clone()
            .into_iter();

        let mut entities = vec![];

        let mut temp_bundle: Option<B> = None;

        let temp_chunk = 'get_chunk: {
            if let Some(cid) = chuns_iter.next() {
                if let Some(chunk) = self.chunks.get_mut(cid) {
                    break 'get_chunk chunk;
                }
            }
            self.new_chunk::<B>()
        };

        let mut eneity_iter = (0..temp_chunk.free()).filter_map(|_| {
            let item = temp_bundle.take().or_else(|| i.next())?;
            temp_chunk
                .insert(item)
                .map_err(|b| temp_bundle = b.into())
                .ok()
        });

        let Some(entity) = eneity_iter.next() else {return entities;};
        entities.extend(std::iter::once(entity).chain(eneity_iter));

        // meta.chunks
        //     .iter()
        //     .filter_map(|&cid| {
        //         let chunk = self.chunks.get_mut(cid)?;
        //         let entitiy_iter = (0..chunk.free()).filter_map(|_| {
        //             let item = temp_bundle.take().or_else(|| i.next())?;
        //             chunk.insert(item).map_err(|b| temp_bundle = Some(b)).ok()
        //         });
        //         entities.extend(entitiy_iter);
        //         Some(())
        //     })
        //     .count();

        // let mut temp_chunk: Option<&mut Chunk> = None;
        // while let Some(b) = i.next().or_else(|| temp_bundle.take()) {
        //     if temp_chunk.is_none() {
        //         temp_chunk = self.new_chunk::<B>().into();
        //     }
        //     let chunk = temp_chunk.as_mut().unwrap();

        //     match chunk.insert(b) {
        //         Ok(entity) => entities.push(entity),
        //         Err(b) => {
        //             temp_chunk = None;
        //             temp_bundle = b.into()
        //         }
        //     }
        // }

        entities
    }

    fn alive(&self, entity: crate::storage::Entity) -> Option<bool> {
        self.chunks.get(entity.index / CHUNK_SIZE)?.alive(entity)
    }

    fn remove(&mut self, entity: crate::storage::Entity) -> bool {
        self.chunks
            .get_mut(entity.index / CHUNK_SIZE)
            .map(|chunk| chunk.remove(entity))
            .unwrap_or(false)
    }

    fn fetch<F: crate::tools::WorldFetch>(&mut self, entity: Entity) -> Option<F::Item<'_>> {
        // 还是不要滥用语法糖
        // let true = self.alive(entity).unwrap_or(false) else{
        //     return None;
        // };

        // 脱糖
        if !self.alive(entity).unwrap_or(false) {
            return None;
        }
        unsafe {
            let chunk = self.chunks.get(entity.chunk_index())?;
            let components = chunk.get(entity.index_in_chunk());
            let mapping_table = self.metas.get_mut(&chunk.bundle_id())?.fetch::<F>()?;
            let item = F::build(components, mapping_table);
            Some(item)
        }
    }
}

impl Drop for World {
    fn drop(&mut self) {
        // Drop资源
        for (t_id, droper) in &mut self.resources_dropers {
            if let (Some(droper), Some(res)) = (droper.take(), self.resources.get_mut(t_id)) {
                (droper)(res);
            }
        }

        // Drop组件
        for (.., meta) in &self.metas {
            meta.chunks
                .iter()
                .copied()
                .filter_map(|cid| {
                    self.chunks.get_mut(cid)?.clear(&meta.droper);
                    Some(())
                })
                .count();
        }
    }
}

impl ResManager for World {
    fn get_res<T: 'static>(&mut self) -> Res<'_, T> {
        if !self.resources.contains_key(&TypeId::of::<T>()) {
            self.new_res::<T>();
        }
        self.try_get_res::<T>().unwrap()
    }

    fn try_get_res<T: 'static>(&mut self) -> Option<Res<'_, T>> {
        let t_id = TypeId::of::<T>();
        let res = self.resources.get_mut(&t_id)?.get_mut();
        Some(Res::new(res))
    }

    fn new_res<T: 'static>(&mut self) {
        Resources {
            resources: &mut self.resources,
            resources_dropers: &mut self.resources_dropers,
        }
        .new_res::<T>();
    }
}

#[cfg(test)]
mod tests {
    use crate::storage::ALIVE_TAG;

    use super::*;

    #[test]
    fn command() {
        let mut world = World::new();

        // 创建一个 然后删除
        let entity = world.spawn(12345);
        world.remove(entity);

        // 新创建的没有立刻覆盖
        assert_eq!(world.spawn(114514).generator, ALIVE_TAG);

        // 创建CHUNK_SIZE-1个 此时刚好填满chunks[0]
        // 最后一个entity因该出现了复用
        let last = *world.spawn_many(1..CHUNK_SIZE as i32).last().unwrap();

        // 具有相同的index 不同的generator
        assert_eq!(entity.index, last.index);
        assert_eq!(entity.generator, last.generator - 1);

        // 此时chunks.len() == 1
        assert_eq!(world.chunks.len(), 1);

        // 新创建一个,因该被放置进chunks[1]
        let entity = world.spawn(12345);
        assert_eq!(world.chunks.len(), 2);
        assert_eq!(entity.index, CHUNK_SIZE);
    }
}
