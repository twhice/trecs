#[allow(unused_imports)]
use crate::{
    bundle::{Bundle, Components},
    storage::Chunk,
};

use super::CHUNK_SIZE;

/// 对[Bundle]生成的[Components]在[World]中的索引
#[derive(Debug, Default, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Entity {
    /// 从[Chunk]的alive数组中拷贝的数据
    ///
    /// 用来计算[Entity]是否有效
    pub(crate) generator: usize,
    /// [Entity]指向的[Bundle]所在的位置
    pub(crate) index: usize,
}

impl Entity {
    pub(crate) fn new(generator: usize, index: usize) -> Self {
        Self { generator, index }
    }

    pub(crate) fn index_in_chunk(&self) -> usize {
        self.index % CHUNK_SIZE
    }
}
