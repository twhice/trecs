#[derive(Debug, Default, Clone, Copy)]
pub struct Entity {
    pub(crate) index: usize,
    pub(crate) generator: usize,
}
