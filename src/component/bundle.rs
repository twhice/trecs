use std::any::{Any, TypeId};

/// 一系列Component的结合
///
/// 可以结构成一些Component
pub trait Bundle: Any {
    /// [Bundle]所有的Component的TypeId
    ///
    /// 为什么Self::SIZE 不是constexpr ? 奇怪
    ///
    /// 这里通过引用外部的[TypeId]数组实现
    const COMPONENT_IDS: &'static [TypeId];

    /// 解构自身为[Vec<Box<dyn Any>>]
    fn deconstruct(self) -> Vec<Box<dyn Any>>;
}

const EMPTY_TYPE_IDS: [TypeId; 0] = [];
impl Bundle for () {
    const COMPONENT_IDS: &'static [TypeId] = &EMPTY_TYPE_IDS;

    fn deconstruct(self) -> Vec<Box<dyn Any>> {
        vec![]
    }
}

pub struct BundleMeta {
    pub components: &'static [TypeId],
    pub bundle_id: TypeId,
    pub chunks: Vec<usize>,
}
