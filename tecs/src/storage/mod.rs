mod entity;
use std::fmt::Debug;

pub use entity::Entity;

use crate::bundle::{Bundle, Components};

/// 一个[Chunk]的大小
///
/// [Chunk]中存放的[Bundle]的数量
pub const CHUNK_SIZE: usize = 1024;
/// 用来标记[Entity]
pub const ALIVE_TAG: usize = 1 << 63;
/// 存放[Bundle]的容器
///
/// + 放入[Bundle]
/// + 复用空间,减少内存分配
/// + 生成[Entity],计算[Entity]有效性
///
pub struct Chunk {
    /// 实际存放[Bundle]的[Vec]
    ///
    /// 长度为[CHUNK_SIZE]
    bundles: Vec<Components>,
    /// 储存对应下标下的[Entity]的信息
    ///
    /// 第一位表示[Entitiy]是否"存活",
    /// 避免在available中进行查找
    ///
    /// 其余位表示这个位置的使用次数
    ///
    /// 如果用来访问的[Entity]和这里存储的不对应,
    /// 说明用来访问的[Entity]是这个位置之前存放的[Bundle]对应的[Entity],
    /// 那么访问的[Entity]是一个失效的[Entity]
    alive: Vec<usize>,
    /// 空闲的位置
    removed: Vec<usize>,
    /// 区块的编号
    ///
    /// 以便直接生成[Entity]
    index: usize,
}

impl Chunk {
    pub fn new(idx: usize) -> Self {
        Self {
            bundles: Vec::with_capacity(CHUNK_SIZE),
            alive: Vec::with_capacity(CHUNK_SIZE),
            removed: vec![],
            index: idx,
        }
    }

    /// 尝试在[Chunk]中插入一个[Bundle]
    ///
    /// + 成功则返回对应的[Entity]
    ///
    /// + 失败则原路返回[Bundle]
    pub fn insert<B: Bundle>(&mut self, b: B) -> Result<Entity, B> {
        if self.bundles.len() != CHUNK_SIZE {
            self.bundles.push(b.destory());
            self.alive.push(ALIVE_TAG);
            return Ok(Entity::new(
                ALIVE_TAG,
                self.index * CHUNK_SIZE + self.bundles.len() - 1,
            ));
        }

        match self.removed.pop() {
            Some(slot) => {
                self.bundles[slot] = b.destory();
                self.alive[slot] += ALIVE_TAG + 1;
                return Ok(Entity::new(
                    self.alive[slot],
                    self.index * CHUNK_SIZE + slot,
                ));
            }
            None => return Err(b),
        }
    }

    /// 从[Chunk]中删除[Entity]对应的[Bundle]
    ///
    /// 返回[Entity]对应的[Bundle]是否存在
    ///
    pub fn remove(&mut self, entity: Entity) -> bool {
        if self.alive(entity).is_none() {
            return false;
        }
        let index = entity.index_in_chunk();
        // 不能remove否则下标会混乱
        self.bundles[index].clear();
        self.alive[index] -= ALIVE_TAG;
        self.removed.push(index);
        true
    }

    /// 计算Entity是否有效
    ///
    /// + 返回[Some(bool)]时,[bool]表示[Entity]是否有效
    ///
    /// + 返回[None]时,表示[Entity]对应的[Bundle]并不存在
    pub fn alive(&self, entity: Entity) -> Option<bool> {
        if entity.index / CHUNK_SIZE != self.index {
            return None;
        }
        let index = entity.index % CHUNK_SIZE;
        Some(*self.alive.get(index)? == entity.generator)
    }

    /// 空闲空间的长度
    pub fn free(&self) -> usize {
        CHUNK_SIZE - self.bundles.len() + self.removed.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert_remove() {
        // hso
        let mut chunk = Chunk::new(0);

        // 先插进去两个
        assert_eq!(chunk.insert(123), Ok(Entity::new(ALIVE_TAG, 0)));
        assert_eq!(chunk.insert(456), Ok(Entity::new(ALIVE_TAG, 1)));

        // 用一些东西填满她剩下的全部空间
        for idx in 2..CHUNK_SIZE {
            assert_eq!(chunk.insert(0), Ok(Entity::new(ALIVE_TAG, idx)))
        }

        // 一点也插不进去了,已经彻底被填满了~
        assert_eq!(chunk.insert(123456), Err(123456));

        // 拔出来一个
        assert_eq!(chunk.remove(Entity::new(ALIVE_TAG, 1)), true);

        // 换成更大的,再插进去
        assert_eq!(chunk.insert(114514), Ok(Entity::new(ALIVE_TAG + 1, 1)))
    }
}

impl Debug for Chunk {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Chunk")
            // 因为Component  不能 Debug
            // .field("bundles", &self.bundles)
            .field("bundles", &"...")
            .field("alive", &self.alive)
            .field("removed", &self.removed)
            .field("index", &self.index)
            .finish()
    }
}
