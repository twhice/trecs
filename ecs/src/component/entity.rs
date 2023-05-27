use crate::world::chunk::CHUNK_SIZE;

#[derive(Debug, Default, Clone, Copy)]
pub struct Entity {
    pub(crate) index: usize,
    pub(crate) generator: usize,
}

impl Entity {
    #[inline]
    pub fn sync(mut self, chunk_id: usize) -> Self {
        self.index += chunk_id * CHUNK_SIZE;
        self
    }
}
