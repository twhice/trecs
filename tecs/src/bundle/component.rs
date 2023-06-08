use std::any::Any;

use super::Bundle;
/// 最基本的构成单元
///
/// 构成[Bundle],并存储在[Chunk]中
///
/// 此特征实际上只是一个标记
///
/// [Bundle]:crate
pub trait Component: Any {}

pub type Components = Vec<Box<dyn Any>>;

// #[rustfmt::skip]
mod __impl {
    use super::{Bundle, Component, Components};
    use std::{
        any::{type_name, Any, TypeId},
        cell::OnceCell,
    };

    macro_rules! impl_components {
        ($($t:ty),*) => {
            $(impl Component for $t{})*
        };
    }

    impl_components!(u8, u16, u32, u64, usize, u128);
    impl_components!(i8, i16, i32, i64, isize, i128);
    impl_components!(bool, (), &'static str);

    macro_rules! impl_bundle {
        ($($t:ident),*) => {
            impl<$($t:Component),*> Bundle for ($($t,)*) {
                fn destory(self) -> Components{
                    let ($($t,)*) = self;
                    vec![
                        $(Box::new($t) as Box<dyn Any>),*
                    ]
                }

                fn components_ids() -> &'static [TypeId] {
                    static mut COMPONENT_ID : OnceCell<Vec<TypeId>> = OnceCell::new();
                    unsafe{
                        COMPONENT_ID.get_or_init(||
                            vec![$(TypeId::of::<$t>(),)*]
                        )
                    }
                }

                fn conponents_name() -> &'static str {
                    type_name::<Self>()
                }
            }
        };
    }

    proc::all_tuple!(impl_bundle, 16);

    impl<C: Component> Bundle for C {
        fn destory(self) -> Components {
            vec![Box::new(self)]
        }

        // 其实直接创快得多
        // 但是为了统一,代价必须有
        // 所以使用static mut

        // OnceCell终于稳定了 真香
        fn components_ids() -> &'static [TypeId] {
            static mut COMPONENT_ID: OnceCell<[TypeId; 1]> = OnceCell::new();
            unsafe { COMPONENT_ID.get_or_init(|| [TypeId::of::<Self>()]) }
        }

        fn conponents_name() -> &'static str {
            type_name::<Self>()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_name() {
        fn print_bundle_meta<B: Bundle>() {
            println!(
                "bundle: {}\ncomponents: {}",
                B::conponents_name(),
                B::components_ids().len(),
            )
        }

        // 编辑器没报错就算是通过了
        print_bundle_meta::<i32>();
        print_bundle_meta::<(i32, usize)>();
        print_bundle_meta::<(&str, usize)>();
    }
}
