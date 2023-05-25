pub(crate) mod chunk;
#[allow(unused)]
pub(crate) mod tree;

use std::{any::TypeId, collections::BTreeMap};

use crate::{
    component::bundle::{Bundle, BundleMeta},
    component::entity::Entity,
};
use chunk::{Chunk, Components, CHUNK_SIZE};

pub struct World {
    /// 区块表
    pub(crate) chunks: Vec<Chunk>,
    /// [Bundle]的元数据
    pub(crate) metas: BTreeMap<TypeId, BundleMeta>,
}

impl World {
    pub fn new() -> Self {
        Self {
            chunks: Default::default(),
            metas: Default::default(),
        }
    }

    /// 给一个实现了[Bundle]的类型注册
    fn register<B: Bundle>(&mut self) -> bool {
        let bundle_id = TypeId::of::<B>();
        if self.metas.get(&bundle_id).is_none() {
            self.metas.insert(
                bundle_id,
                BundleMeta {
                    components: B::conponents_ids(),
                    bundle_id,
                    chunks: vec![],
                },
            );
            false
        } else {
            true
        }
    }

    /// 将一个实现了[Bundle]的类型放入世界
    pub fn spawn<B: Bundle>(&mut self, b: B) -> Entity {
        self.register::<B>();
        let bundle_id = TypeId::of::<B>();
        let meta = self.metas.get_mut(&bundle_id).unwrap();
        let mut vul = Some(b.deconstruct());

        for chunk_id in &meta.chunks {
            let chunk = &mut self.chunks[*chunk_id];
            match chunk.insert(vul.take().unwrap()) {
                Ok(mut entity) => {
                    entity.index += *chunk_id * CHUNK_SIZE;
                    return entity;
                }
                Err(v) => vul = Some(v),
            }
        }

        {
            let mut chunk = Chunk::new::<B>();
            let mut entity = chunk.insert(vul.unwrap()).unwrap();
            entity.index += self.chunks.len() * CHUNK_SIZE;
            meta.chunks.push(self.chunks.len());

            self.chunks.push(chunk);
            entity
        }
    }

    /// 插入迭代器
    ///
    /// 对于大量数据 性能更好
    pub fn spawn_iter<B: Bundle, I: IntoIterator<Item = B>>(&mut self, iter: I) -> Vec<Entity> {
        let mut iter = iter.into_iter();
        self.register::<B>();
        let bundle_id = TypeId::of::<B>();
        let meta = self.metas.get_mut(&bundle_id).unwrap();

        let chunks = meta.chunks.clone();
        let mut chunks = chunks.into_iter();
        // 断言: 至少有一个
        let mut chunk_id = chunks.next().unwrap();
        let mut chunk = &mut self.chunks[chunk_id];

        let mut entities = vec![];

        let insert =
            |cs: Components, chunk_id: usize, chunk: &mut Chunk| -> Result<Entity, Components> {
                let mut entity = chunk.insert(cs)?;
                entity.index += chunk_id * CHUNK_SIZE;
                Ok(entity)
            };

        while let Some(bundle) = iter.next() {
            let mut cs = Some(bundle.deconstruct());
            loop {
                match insert(cs.take().unwrap(), chunk_id, chunk) {
                    Ok(entity) => {
                        entities.push(entity);
                        break;
                    }
                    Err(cs_) => {
                        cs = Some(cs_);
                        if let Some(cid) = chunks.next() {
                            chunk_id = cid;
                        } else {
                            chunk_id = self.chunks.len();
                            meta.chunks.push(chunk_id);
                            self.chunks.push(Chunk::new::<B>());
                        }
                        chunk = &mut self.chunks[chunk_id];
                    }
                }
            }
        }
        entities
    }

    /// 快速插入 但是不是泛型
    ///
    /// 实际上为对Commands的接口
    pub fn spwan_iter_boxed<
        I: IntoIterator<Item = (TypeId, &'static [TypeId], Vec<Components>)>,
    >(
        &mut self,
        iter: I,
    ) {
        let mut entities = vec![];
        for (bundle_id, components, bundles) in iter {
            // 筹备区块 & 元数据
            let meta = {
                let nr_chunks = bundles.len() / CHUNK_SIZE
                    + if bundles.len() % CHUNK_SIZE != 0 {
                        1
                    } else {
                        0
                    };
                if !self.metas.contains_key(&bundle_id) {
                    self.metas.insert(
                        bundle_id,
                        BundleMeta {
                            components,
                            bundle_id,
                            chunks: (0..nr_chunks).map(|n| self.chunks.len() + n).collect(),
                        },
                    );
                    for _ in 0..nr_chunks {
                        self.chunks.push(Chunk::create(bundle_id, components));
                    }
                }
                let meta = self.metas.get_mut(&bundle_id).unwrap();
                if meta.chunks.len() < nr_chunks {
                    for _ in 0..(nr_chunks - meta.chunks.len()) {
                        meta.chunks.push(self.chunks.len());
                        self.chunks.push(Chunk::create(bundle_id, components));
                    }
                }
                meta
            };
            let mut chunk_id_id = 0;
            let mut chunk_id = meta.chunks[chunk_id_id];
            let mut chunk = &mut self.chunks[chunk_id];
            for cs in bundles {
                let mut cs = Some(cs);
                loop {
                    match chunk.insert(cs.take().unwrap()) {
                        Ok(mut entity) => {
                            entity.index += chunk_id * CHUNK_SIZE;
                            entities.push(entity);
                            break;
                        }
                        Err(cs_) => {
                            cs = Some(cs_);
                            chunk_id_id += 1;
                            chunk_id = meta.chunks[chunk_id_id];
                            chunk = &mut self.chunks[chunk_id];
                        }
                    }
                }
            }
        }
    }
    #[inline]
    fn locate(entity: Entity) -> (usize, usize) {
        (entity.index / CHUNK_SIZE, entity.index % CHUNK_SIZE)
    }

    /// 检测entity是否有效
    ///
    /// 返回[None]表示entity不存在
    #[inline]
    pub fn alive(&self, entity: Entity) -> Option<bool> {
        let (chunk_id, _) = Self::locate(entity);
        self.chunks.get(chunk_id)?.alive(entity)
    }

    /// 删除一个[Entity]
    ///
    /// 返回entity是否存在
    pub fn remove(&mut self, entity: Entity) -> bool {
        let mut foo = || {
            if !self.alive(entity)? {
                return Some(false);
            }
            let (chid, coid) = Self::locate(entity);
            Some(self.chunks[chid].remove(coid))
        };
        foo().unwrap_or(false)
    }

    /// 将[Entity]从[World]中移出
    ///
    /// 如果entity不存在, 返回[None]
    pub fn remove_vul(&mut self, entity: Entity) -> Option<Components> {
        if !self.alive(entity)? {
            return None;
        }
        let (chid, coid) = Self::locate(entity);
        Some(self.chunks[chid].remove_vul(coid))
    }

    /// 覆盖掉[Entity]
    ///
    /// 如果失败会返回Err([Bundle])
    pub fn replace<B: Bundle>(&mut self, b: B, entity: Entity) -> Result<(), B> {
        if !self.alive(entity).unwrap_or(false) {
            return Err(b);
        }

        let (chid, coid) = Self::locate(entity);
        let chunk: &mut Chunk = &mut self.chunks[chid];

        if chunk.meta.1 != B::conponents_ids() {
            return Err(b);
        }

        let v = b.deconstruct();
        chunk.replace(coid, v);
        Ok(())
    }

    /// 覆盖掉[Entity]
    ///
    /// 如果成功会返回Ok([Components])
    ///
    /// 如果失败会返回Err([Bundle])
    pub fn replace_vul<B: Bundle>(&mut self, b: B, entity: Entity) -> Result<Components, B> {
        if !self.alive(entity).unwrap_or(false) {
            return Err(b);
        }

        let (chid, coid) = Self::locate(entity);
        let chunk: &mut Chunk = &mut self.chunks[chid];

        if chunk.meta.1 != B::conponents_ids() {
            return Err(b);
        }

        let v = b.deconstruct();
        let c = chunk.remove_vul(coid);
        chunk.replace(coid, v);
        Ok(c)
    }
}

impl Default for World {
    fn default() -> Self {
        Self::new()
    }
}
