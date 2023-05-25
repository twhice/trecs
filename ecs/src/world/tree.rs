use std::any::TypeId;

pub const TREE_SIZE: usize = 4;

/// 一种特殊的数据结构
///
/// 在这里专门用来储存[TypeId]这种纯数字数据
///
/// 通过位运算将u64分为多个ux,根据ux作为索引逐层索引
///
/// 来做到快速(?)索引
///
/// 效率比之哈希表如何? 不知
pub enum TypeIdTree<T> {
    Node {
        /// 节点
        ///
        /// 外层用[Box]是为了让类型更小
        nodes: Vec<Option<Box<TypeIdTree<T>>>>,
    },
    Data {
        vul: T,
    },
}

fn typeid_to_u64(id: TypeId) -> u64 {
    unsafe { std::mem::transmute(id) }
}

impl<T> TypeIdTree<T> {
    pub fn new() -> TypeIdTree<T> {
        let mut nodes = Vec::with_capacity(1 << TREE_SIZE);
        for _ in 0..1 << TREE_SIZE {
            nodes.push(None)
        }
        Self::Node { nodes }
    }

    #[inline]
    pub fn set(&mut self, index: TypeId, v: T) -> bool {
        let index = typeid_to_u64(index);
        if let Some(data) = self.index_to_or_create(index) {
            *data = Self::Data { vul: v };
            true
        } else {
            false
        }
    }

    #[inline]
    pub fn get_mut(&mut self, index: TypeId) -> Option<&mut T> {
        let index = typeid_to_u64(index);
        self.index_to_or_create(index)?.as_data_mut()
    }

    pub fn get(&self, index: TypeId) -> Option<&T> {
        let index = typeid_to_u64(index);
        let try_get = |nodes: &Self, index: usize| -> Option<&Self> {
            let opt = &nodes.as_node()?[index];
            if opt.is_none() {
                None
            } else {
                Some(unsafe { &mut *(opt as *const _ as *mut Box<Self>) })
            }
        };

        let mut nodes = self;

        let mut empty_bits = 64 - TREE_SIZE;
        let mut index_bits = (1 << TREE_SIZE) - 1 << empty_bits;
        let mut part_of_index = (index & index_bits) >> empty_bits;

        while empty_bits != 0 {
            nodes = try_get(nodes, part_of_index as usize)?;

            empty_bits -= TREE_SIZE;
            index_bits >>= TREE_SIZE;
            part_of_index = (index & index_bits) >> empty_bits;
        }

        nodes.as_data()
    }

    fn index_to_or_create(&mut self, index: u64) -> Option<&mut Self> {
        let mut nodes = self;

        let mut empty_bits = 64 - TREE_SIZE;
        let mut index_bits = (1 << TREE_SIZE) - 1 << empty_bits;
        let mut part_of_index = (index & index_bits) >> empty_bits;

        while empty_bits != 0 {
            nodes = Self::get_or_recreate(nodes, part_of_index as usize)?;

            empty_bits -= TREE_SIZE;
            index_bits >>= TREE_SIZE;
            part_of_index = (index & index_bits) >> empty_bits;
        }
        Some(nodes)
    }

    fn get_or_recreate(nodes: &mut Self, index: usize) -> Option<&mut Self> {
        let opt = &mut nodes.as_node_mut()?[index];
        if opt.is_none() {
            *opt = Some(Box::new(Default::default()));
        }
        Some(unsafe { &mut *(opt as *const _ as *mut Box<Self>) })
    }

    pub fn as_node_mut(&mut self) -> Option<&mut Vec<Option<Box<TypeIdTree<T>>>>> {
        if let Self::Node { ref mut nodes } = self {
            Some(nodes)
        } else {
            None
        }
    }

    pub fn as_data_mut(&mut self) -> Option<&mut T> {
        if let Self::Data { vul } = self {
            Some(vul)
        } else {
            None
        }
    }

    pub fn as_node(&self) -> Option<&Vec<Option<Box<TypeIdTree<T>>>>> {
        if let Self::Node { nodes } = self {
            Some(nodes)
        } else {
            None
        }
    }

    pub fn as_data(&self) -> Option<&T> {
        if let Self::Data { vul } = self {
            Some(vul)
        } else {
            None
        }
    }
}

impl<T> Default for TypeIdTree<T> {
    fn default() -> Self {
        Self::new()
    }
}
