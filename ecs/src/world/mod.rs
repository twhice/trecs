pub(crate) mod chunk;
pub(crate) mod tree;

use std::any::TypeId;

use crate::{
    component::bundle::{Bundle, BundleMeta},
    component::entity::Entity,
};
use chunk::{Chunk, Components, CHUNK_SIZE};
use tree::TypeIdTree;

pub struct World {
    chunks: Vec<Chunk>,
    metas: TypeIdTree<BundleMeta>,
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
        if self.metas.get(bundle_id).is_none() {
            self.metas.set(
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
        let meta = self.metas.get_mut(bundle_id).unwrap();
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
