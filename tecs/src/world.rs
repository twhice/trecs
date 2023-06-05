use std::{any::TypeId, collections::HashMap};

use crate::{
    bundle::{Bundle, BundleMeta},
    storage::{Chunk, Entity, CHUNK_SIZE},
    traits::command::Command,
};

#[derive(Debug)]
pub struct World {
    pub(crate) chunks: Vec<Chunk>,
    pub(crate) metas: HashMap<TypeId, BundleMeta>,
}

impl World {
    pub fn new() -> Self {
        Self {
            chunks: vec![],
            metas: Default::default(),
        }
    }

    /// 创建一个新的区块,并且返回它的可变引用
    ///
    /// 防止诸如"meta和实际不一致","chunk.index不正确"等错位问题
    pub(crate) fn new_chunk<B: Bundle>(&mut self) -> &mut Chunk {
        self.metas
            .get_mut(&TypeId::of::<B>())
            .unwrap()
            .chunks
            .push(self.chunks.len());
        self.chunks.push(Chunk::new(self.chunks.len()));
        self.chunks.last_mut().unwrap()
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
        let bundle_id = TypeId::of::<B>();
        if !self.metas.contains_key(&bundle_id) {
            self.metas.insert(bundle_id, BundleMeta::new::<B>());
        }
    }

    fn spawn<B: crate::bundle::Bundle>(&mut self, b: B) -> crate::storage::Entity {
        self.register::<B>();
        let bundle_id = TypeId::of::<B>();
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
        let meta = self.metas.get_mut(&TypeId::of::<B>()).unwrap();

        // 准备迭代器和返回
        let mut i = i.into_iter();
        let mut entities = vec![];

        let mut temp: Option<B> = None;

        meta.chunks
            .iter()
            .filter_map(|&cid| {
                let chunk = self.chunks.get_mut(cid)?;
                let entitiy_iter = (0..chunk.free()).filter_map(|_| {
                    let item = temp.take().or_else(|| i.next())?;
                    chunk.insert(item).or_else(|b| Err(temp = Some(b))).ok()
                });
                Some(entities.extend(entitiy_iter))
            })
            .count();

        let mut temp_chunk: Option<&mut Chunk> = None;
        while let Some(b) = i.next().or_else(|| temp.take()) {
            if temp_chunk.is_none() {
                temp_chunk = self.new_chunk::<B>().into();
            }
            let chunk = temp_chunk.as_mut().unwrap();

            match chunk.insert(b) {
                Ok(entity) => entities.push(entity),
                Err(b) => {
                    temp_chunk = None;
                    temp = b.into()
                }
            }
        }

        entities
    }

    fn alive(&self, entity: crate::storage::Entity) -> Option<bool> {
        self.chunks.get(entity.index / CHUNK_SIZE)?.alive(entity)
    }

    fn remove(&mut self, entity: crate::storage::Entity) -> bool {
        self.chunks
            .get_mut(entity.index / CHUNK_SIZE)
            .and_then(|chunk| Some(chunk.remove(entity)))
            .unwrap_or(false)
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
        assert_eq!(entity.index, CHUNK_SIZE + 0);
    }
}
