pub(crate) mod chunk;
pub(crate) mod traits;
#[allow(unused)]
pub(crate) mod tree;

use std::{
    any::{Any, TypeId},
    collections::{BTreeMap, HashMap},
};

use crate::{component::bundle::BundleMeta, component::entity::Entity, Scheduler};
use chunk::{Chunk, Components, CHUNK_SIZE};

use self::traits::{InnerCommand, InnerResources};

pub struct World {
    /// 区块表
    pub(crate) chunks: Vec<Chunk>,
    /// [Bundle]的元数据
    pub(crate) metas: BTreeMap<TypeId, BundleMeta>,

    pub(crate) resources: HashMap<TypeId, Box<dyn Any>>,
}

impl World {
    pub fn new() -> Self {
        Self {
            chunks: Default::default(),
            metas: Default::default(),
            resources: Default::default(),
        }
    }

    #[inline]
    fn locate(entity: Entity) -> (usize, usize) {
        (entity.index / CHUNK_SIZE, entity.index % CHUNK_SIZE)
    }

    /// 创建调度表
    #[inline]
    pub fn scheduler(&mut self) -> Scheduler<'_> {
        Scheduler::from_world(self)
    }
}

impl Default for World {
    fn default() -> Self {
        Self::new()
    }
}

impl InnerCommand for World {
    fn remove(&mut self, entity: Entity) -> bool {
        let mut foo = || {
            if !self.alive(entity)? {
                return Some(false);
            }
            let (chid, coid) = Self::locate(entity);
            Some(self.chunks[chid].remove(coid))
        };
        foo().unwrap_or(false)
    }

    fn r#return(&mut self, entity: Entity) -> Option<Components> {
        if !self.alive(entity)? {
            return None;
        }
        let (chid, coid) = Self::locate(entity);
        Some(self.chunks[chid].remove_vul(coid))
    }

    fn alive(&self, entity: Entity) -> Option<bool> {
        let (chunk_id, _) = Self::locate(entity);
        self.chunks.get(chunk_id)?.alive(entity)
    }

    fn inner_register(&mut self, bundle_id: TypeId, components: &'static [TypeId]) -> bool {
        if !self.metas.contains_key(&bundle_id) {
            let meta = BundleMeta {
                components,
                bundle_id,
                chunks: vec![],
            };
            self.metas.insert(bundle_id, meta);
            false
        } else {
            true
        }
    }

    fn inner_spawn(
        &mut self,
        bundle_id: TypeId,
        components: &'static [TypeId],
        bundle: Components,
    ) -> Entity {
        let mut bundle = Some(bundle);
        self.inner_register(bundle_id, components);
        for chunk_id in self.metas.get(&bundle_id).unwrap().chunks.iter() {
            let chunk = &mut self.chunks[*chunk_id];
            bundle = match chunk.insert(bundle.take().unwrap()) {
                Ok(entity) => return entity.sync(*chunk_id),
                Err(b) => Some(b),
            }
        }
        {
            let mut chunk = Chunk::create(bundle_id, components);
            let entity = chunk
                .insert(bundle.unwrap())
                .unwrap()
                .sync(self.chunks.len());
            self.metas
                .get_mut(&bundle_id)
                .unwrap()
                .chunks
                .push(self.chunks.len());
            self.chunks.push(chunk);
            entity
        }
    }

    fn inner_spawn_many(
        &mut self,
        bundle_id: TypeId,
        components: &'static [TypeId],
        bundles: Vec<Components>,
    ) -> Vec<Entity> {
        self.inner_register(bundle_id, components);
        let mut entities = vec![];
        let mut bundles = bundles.into_iter();
        // 优先尝试填满已经分配的空间
        for chunk_id in self.metas.get(&bundle_id).unwrap().chunks.iter() {
            let chunk = &mut self.chunks[*chunk_id];
            // 区块没有满 并且bundle还有剩余
            while let (true, Some(item)) = (chunk.len() != CHUNK_SIZE, bundles.next()) {
                entities.push(chunk.insert(item).unwrap().sync(*chunk_id));
            }
        }

        let meta = self.metas.get_mut(&bundle_id).unwrap();
        let mut chunk_id = self.chunks.len();

        loop {
            // 创建新区块 并且不断写入
            let mut chunk = Chunk::create(bundle_id, components);

            // 直到chunk已满 / bundles没有剩余
            while let (true, Some(item)) = (chunk.len() != CHUNK_SIZE, bundles.next()) {
                entities.push(chunk.insert(item).unwrap().sync(chunk_id));
            }

            // 更新chunk_id
            chunk_id = self.chunks.len();

            // 如果Chunk没有被填满,说明Bundle已经全部放进Chunk中
            // 应当Break
            let should_break = chunk.len() != CHUNK_SIZE;
            // 将Chunk放入meta,放入区块表
            meta.chunks.push(chunk_id);
            self.chunks.push(chunk);

            if should_break {
                break;
            }
        }

        entities
    }
}

impl InnerResources for World {
    fn inner_get_res_mut(&mut self, resources_id: TypeId) -> Option<&mut Box<dyn Any>> {
        self.resources.get_mut(&resources_id)
    }

    fn inner_get_res(&self, resources_id: TypeId) -> Option<&Box<dyn Any>> {
        self.resources.get(&resources_id)
    }
    fn inner_insert(
        &mut self,
        resources_id: TypeId,
        resources: Box<dyn Any>,
    ) -> Option<Box<dyn Any>> {
        self.resources.insert(resources_id, resources)
    }

    fn inner_contain(&self, resources_id: TypeId) -> bool {
        self.resources.contains_key(&resources_id)
    }
}
