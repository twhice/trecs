use crate::{bundle::Components, storage::ALIVE_TAG};

use super::Chunk;

pub struct ChunkIter<'a> {
    pub(crate) chunk: &'a Chunk,
    pub(crate) index: usize,
    pub(crate) first: bool,
}

impl ChunkIter<'_> {
    pub fn new(chunk: &'_ Chunk) -> ChunkIter<'_> {
        ChunkIter {
            chunk,
            index: 0,
            first: true,
        }
    }
}

impl<'a> Iterator for ChunkIter<'a> {
    type Item = &'a Components;

    fn next(&mut self) -> Option<Self::Item> {
        if !self.first {
            self.index += 1
        } else {
            self.first = false;
        }
        while self.chunk.alive.get(self.index).copied()? < ALIVE_TAG {}
        Some(&self.chunk.bundles[self.index])
    }
}
