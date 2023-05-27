use std::{
    any::{Any, TypeId},
    collections::{BTreeMap, BTreeSet},
    marker::PhantomData,
};

use crate::world::World;

use self::{
    access::Access,
    caches::{FetchCache, QueryCache},
    command::Command,
    fetch::{MappingTable, WorldFetch},
    query::WorldQuery,
    resources::Resources,
    system::{System, SystemState},
};

pub(crate) mod access;
pub(crate) mod caches;
pub(crate) mod command;
pub(crate) mod fetch;
pub(crate) mod iter;
pub(crate) mod query;
pub(crate) mod resources;
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

/// ### 调度器
///    
/// 协调,加快[System]的运行,以及进进行一些检查
///
/// * 缓存[WorldFetch],[WorldQuery]
/// 对[Bundle]的结果来加快运行速度,减少[System]运行之前的准备时间
///
/// * 检查[Resources]和[Res]的重复,避免别名
///
/// * 根据缓存创建[Res],[Resources],[Access]和[Command]
///
/// [Bundle]:crate
/// [Res]: crate
pub struct Scheduler<'a> {
    /// 世界的副本
    world: &'a mut World,

    cached: BTreeSet<TypeId>,
    fetch_caches: BTreeMap<TypeId, FetchCache>,
    query_caches: BTreeMap<TypeId, QueryCache>,

    systems: Vec<Box<dyn System<'a>>>,

    temp_system_state: SystemState,
}

impl<'a> Scheduler<'a> {
    pub(crate) fn from_world(world: &mut World) -> Scheduler<'_> {
        Scheduler {
            world,
            cached: Default::default(),
            fetch_caches: Default::default(),
            query_caches: Default::default(),
            systems: Default::default(),
            temp_system_state: Default::default(),
        }
    }

    /// 注册[WorldFetch],实际计算在sync函数中
    pub(crate) fn registry_fetch<F: WorldFetch>(&mut self) {
        let fetch_id = TypeId::of::<F>();
        if !self.fetch_caches.contains_key(&fetch_id) {
            self.fetch_caches.insert(fetch_id, FetchCache::new::<F>());
        }
    }

    /// 注册[WorldQuery],实际计算在sync函数中
    pub(crate) fn registry_query<Q: WorldQuery>(&mut self) {
        let query_id = TypeId::of::<Q>();
        if !self.query_caches.contains_key(&query_id) {
            self.query_caches.insert(query_id, QueryCache::new::<Q>());
        }
    }

    /// 给尚未缓存[WorldFetch::contain]和[WorldQuery::pass]的[Bundle]进行缓存
    ///
    /// (目前)需要在所有[System]准备完毕后才可以运行
    /// 否则部分[Bundle]可能对于部分[WorldFetch::contain]和[WorldQuery::pass]
    /// 没有缓存,从而导致bug
    ///
    /// [Bundle]: crate
    pub(crate) fn sync(&mut self) {
        for (bundle_id, meta) in &self.world.metas {
            if self.cached.contains(&bundle_id) {
                continue;
            }

            for (_, query_cache) in &mut self.query_caches {
                let pass = (query_cache.pass)(meta.components);
                query_cache.tables.insert(*bundle_id, pass);
            }
            for (_, fetch_cache) in &mut self.fetch_caches {
                let table = (fetch_cache.contain)(&mut meta.components.to_vec());
                fetch_cache.tables.insert(*bundle_id, table);
            }

            self.cached.insert(*bundle_id);
        }
    }

    /// 创建[Access]
    pub(crate) fn new_access<F: WorldFetch, Q: WorldQuery>(&self) -> Access<'_, F, Q> {
        let fetch_id = TypeId::of::<F>();
        let query_id = TypeId::of::<Q>();

        let fetch_cache = self.fetch_caches.get(&fetch_id).unwrap();

        let selected_chunks = self
            .query_caches
            .get(&query_id)
            .unwrap()
            .tables
            .iter()
            .filter(|(_, pass)| **pass)
            .filter_map(|(bundle_id, _)| Some((*bundle_id, fetch_cache.tables.get(bundle_id)?)))
            .filter_map(|(bundle_id, mapping)| Some((bundle_id, mapping.as_ref()?)))
            .map(|(bundle_id, mapping_table)| {
                let chunks = self.world.metas.get(&bundle_id).unwrap().chunks.clone();
                (mapping_table, chunks)
            })
            .collect::<Vec<(&MappingTable, Vec<usize>)>>();
        Access::<F, Q> {
            selected_chunks,
            chunks: &self.world.chunks,
            _ph: PhantomData,
        }
    }

    /// 创建[Command]
    pub(crate) fn new_command(&self) -> Command {
        Command::new::<World>(unsafe { transmute_ref(self.world) })
    }

    /// 创建[Access]
    pub(crate) fn new_resources(&self) -> Resources {
        Resources::new::<World>(unsafe { transmute_ref(self.world) })
    }

    /// 在调度器中加入一个新的[System],并且准备缓存
    ///
    /// 系统运行的顺序是加入系统的顺序
    pub fn add_system<S: System<'a> + 'static>(&'a mut self, mut system: S) -> &mut Self {
        self.temp_system_state = Default::default();
        system.init(self);
        self.systems.push(Box::new(system));
        self
    }

    /// 给[World]加入一个[Res]
    ///
    /// 以便被[System]的[Resources]和[Res]选中
    ///
    /// 访问不存在的[Res]会导致painc
    ///
    /// 但是通过[Resources]则不会
    ///
    /// [Res]: crate
    pub fn add_resources<R: Any>(&mut self, r: R) -> &mut Self {
        self.world.resources.insert(TypeId::of::<R>(), Box::new(r));
        self
    }

    /// 生成[Schedule]
    ///
    /// 从此开始不能再加入新的系统
    pub fn build(mut self) -> Schedule<'a> {
        let mut systems = Vec::with_capacity(self.systems.len());
        systems.append(&mut self.systems);

        Schedule {
            inner: self,
            systems,
        }
    }
}

/// 调度器的结果
///
/// 所有[System]和[Resources]全部就绪,可以开始运行
pub struct Schedule<'a> {
    inner: Scheduler<'a>,
    systems: Vec<Box<dyn System<'a>>>,
}

impl<'a> Schedule<'a> {
    /// 运行一次所有系统
    pub fn run(&mut self) {
        let this = unsafe { transmute_lifetime(self) };
        this.inner.sync();

        for sys in &mut self.systems {
            sys.run_once(&this.inner);
        }
    }
}
