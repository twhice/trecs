use std::any::{Any, TypeId};

pub use tecs_proc::Bundle;

use super::Component;

/// 一系列Component的结合
///
/// 可以结构成一些Component
pub trait Bundle: Any {
    /// [Bundle]所有的Component的TypeId
    ///
    fn conponents_ids() -> &'static [TypeId];

    /// 解构自身为[Vec<Box<dyn Any>>]
    fn deconstruct(self) -> Vec<Box<dyn Any>>;
}

pub struct BundleMeta {
    pub components: &'static [TypeId],
    pub bundle_id: TypeId,
    pub chunks: Vec<usize>,
}

impl<C: Component> Bundle for C {
    #[inline]
    fn conponents_ids() -> &'static [TypeId] {
        static mut TYPE_ID: Option<TypeId> = None;
        unsafe {
            if TYPE_ID.is_none() {
                TYPE_ID = Some(TypeId::of::<C>());
            }
            &*(&TYPE_ID as *const _ as *const [TypeId; 1])
        }
    }

    #[inline]
    fn deconstruct(self) -> Vec<Box<dyn Any>> {
        vec![Box::new(self)]
    }
}

tecs_proc::derive_bundle_for_tuple!();

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_name() {
        assert_eq!(
            <(usize, &str) as Bundle>::conponents_ids(),
            &[TypeId::of::<usize>(), TypeId::of::<&str>()]
        );
    }
}
