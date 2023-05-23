use crate::component::bundle::Bundle;
use crate::component::entity::Entity;
use std::any::{Any, TypeId};

pub const CHUNK_SIZE: usize = 1024;
pub const ALIVE_FLAG: usize = 1 << 63;

pub type Components = Vec<Box<dyn Any>>;

/// 用来储存[Components]的容器
///
/// 可以储存[CHUNK_SIZE]个[Components]
///
/// 内部没有任何检查 可能发生越界
///
/// 所有检查都交给[World]
///
/// [World]: crate::world::World
#[derive(Debug)]
pub(crate) struct Chunk {
    /// 实际上的容器
    ///
    /// 每个[Bundle]为一个[Components]
    storage: Vec<Vec<Box<dyn Any>>>,
    /// 用来对应位置的组件是否还有效
    ///
    /// 第一位是组建是否被移除
    ///
    /// 其余位是此索引的使用次数
    alive: Vec<usize>,
    /// 缓存被移除的部分
    removed: Vec<usize>,
    /// [Bundle]的TypeId和Components的类型
    pub meta: (TypeId, &'static [TypeId]),
}

impl Chunk {
    pub fn new<B: Bundle>() -> Self {
        Self {
            storage: Vec::with_capacity(CHUNK_SIZE),
            alive: Vec::with_capacity(CHUNK_SIZE),
            removed: vec![],
            meta: (TypeId::of::<B>(), &B::COMPONENT_IDS),
        }
    }

    // 失败后返回Err(v),也就是原来的值
    pub fn insert(&mut self, v: Components) -> Result<Entity, Components> {
        let index = if let Some(index) = self.removed.pop() {
            // 复用空间
            self.storage[index] = v;
            // 第一位设置为1
            self.alive[index] += ALIVE_FLAG;
            index
        } else if self.storage.len() != CHUNK_SIZE {
            // 新空间
            self.storage.push(v);
            self.alive.push(ALIVE_FLAG);
            self.storage.len() - 1
        } else {
            // 没有空间
            return Err(v);
        };
        // "生成次数"加一
        self.alive[index] += 1;
        let entity = Entity {
            index,
            generator: self.alive[index],
        };

        Ok(entity)
    }

    /// 删除并且返还数据
    pub fn remove_vul(&mut self, index: usize) -> Components {
        // 设置为removed
        self.alive[index] -= ALIVE_FLAG;
        self.removed.push(index);

        // swap调转被移除和最后一个的位置
        // 如果已经满了 就直接swap
        // 如果还没有满 push一个作为被移除位置的新数据
        let storage_len = self.storage.len();
        if storage_len != CHUNK_SIZE {
            self.storage.push(Vec::with_capacity(self.meta.1.len()));
        }
        self.storage.swap(index, storage_len - 1);
        let result = self.storage.pop().unwrap();
        // 补回被移除的数据
        self.storage.push(Vec::with_capacity(self.meta.1.len()));
        // 如果本来是满的 交换后最后一位的位置就不对
        // 需要swap回来
        if storage_len != CHUNK_SIZE {
            self.storage.swap(index, storage_len - 1);
        }
        result
    }

    /// 只是删除
    #[inline]
    pub fn remove(&mut self, index: usize) -> bool {
        self.removed.push(index);
        self.alive[index] -= ALIVE_FLAG;
        true
    }

    /// 检测Entity是否有效
    ///
    /// 这个是完全的检测
    ///
    /// 因为会进行取余 所以不需要对entity做额外操作
    ///
    /// [None]表示Entity不存在
    pub fn alive(&self, entity: Entity) -> Option<bool> {
        self.alive
            .get(entity.index % CHUNK_SIZE)
            .and_then(|alive| Some(entity.generator == *alive))
    }

    #[inline]
    /// 覆盖一个元素
    pub fn replace(&mut self, index: usize, v: Components) {
        self.storage[index] = v;
        self.alive[index] += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chunk() {
        // 实际上不能这样做
        // 只是一个绕过
        let mut chunk = Chunk::new::<()>();

        {
            let bundle = vec![
                Box::new(1) as Box<dyn Any>,
                Box::new("hello world") as Box<dyn Any>,
            ];
            chunk.insert(bundle).unwrap();
        }
        {
            let bundle = vec![
                Box::new(1) as Box<dyn Any>,
                Box::new("hello world") as Box<dyn Any>,
            ];
            chunk.insert(bundle).unwrap();
        }
        dbg!(&chunk);
        chunk.remove(0);
        {
            let bundle = vec![
                Box::new(1) as Box<dyn Any>,
                Box::new("hello world") as Box<dyn Any>,
            ];
            chunk.insert(bundle).unwrap();
        }
        dbg!(&chunk);
    }
}
