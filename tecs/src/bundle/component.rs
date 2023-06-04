use std::any::{Any, TypeId};

use super::Bundle;
/// 最基本的构成单元
///
/// 构成[Bundle],并存储在[Chunk]中
///
/// 此特实际上只是一个标记
///
/// [Bundle]:crate
pub trait Component: Any {}

pub type Components = Vec<Box<dyn Component>>;

impl<C: Component> Bundle for C {
    fn destory(self) -> Components {
        vec![Box::new(self)]
    }

    // 其实直接创快得多
    // 但是为了统一,代价必须有
    // 所以使用static mut
    fn components_ids(&self) -> &[std::any::TypeId] {
        static mut COMPONENT_ID: Option<[TypeId; 1]> = None;
        unsafe {
            if COMPONENT_ID.is_none() {
                COMPONENT_ID = Some([TypeId::of::<Self>()]);
            }
            &*(&COMPONENT_ID as *const _ as *const [TypeId; 1])
        }
    }
}

#[rustfmt::skip]
mod __impl{
    use super::Component;
    macro_rules! impl_components {
        ($($t:ty),*) => {
            $(impl Component for $t{})*
        };
    }

    impl_components!(u8,u16,u32,u64,usize,u128);
    impl_components!(i8,i16,i32,i64,isize,i128);
    impl_components!(bool,(),&'static str);
}
