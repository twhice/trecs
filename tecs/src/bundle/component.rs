use std::any::{Any, TypeId};

use super::Bundle;
/// 最基本的构成单元
///
/// 构成[Bundle],并存储在[Chunk]中
///
/// 此特征实际上只是一个标记
///
/// [Bundle]:crate
pub trait Component: Any {
    fn type_id_() -> TypeId;
}

pub type Components = Vec<Box<dyn Any>>;

// #[rustfmt::skip]
mod __impl {
    use super::{Bundle, Component, Components};
    use std::{
        any::{type_name, Any, TypeId},
        cell::OnceCell,
        collections::HashMap,
    };

    macro_rules! impl_components {
        ($($t:ty),*) => {
            $(impl Component for $t{
                fn type_id_() -> TypeId{
                    TypeId::of::<Self>()
                }
            })*
        };
    }

    impl_components!(u8, u16, u32, u64, usize, u128);
    impl_components!(i8, i16, i32, i64, isize, i128);
    impl_components!(bool, (), &'static str);

    impl<C: Component> Component for &'static C {
        fn type_id_() -> TypeId {
            TypeId::of::<Self>()
        }
    }
    impl<C: Component> Component for &'static mut C {
        fn type_id_() -> TypeId {
            TypeId::of::<Self>()
        }
    }

    macro_rules! impl_bundle {
        ($($t:ident),*) => {
            impl<$($t:Bundle),*> Bundle for ($($t,)*) {
                fn destory(self) -> Components{
                    let ($($t,)*) = self;
                    vec![
                        $(Box::new($t) as Box<dyn Any>),*
                    ]
                }

                fn components_ids() -> &'static [TypeId] {
                    static mut COMPONENT_IDS: OnceCell<HashMap<TypeId, Vec<TypeId>>> = OnceCell::new();
                    unsafe {
                        COMPONENT_IDS.get_or_init(|| HashMap::new());
                        let hashset = COMPONENT_IDS.get_mut().unwrap();
                        if !hashset.contains_key(&Self::type_id_()) {
                            hashset.insert(Self::type_id_(), vec![$($t::type_id_(),)*]);
                        }
                        hashset.get(&Self::type_id_()).unwrap()
                    }
                }

                fn type_name() -> &'static str {
                    type_name::<Self>()
                }

                fn type_id_() -> TypeId{
                    TypeId::of::<Self>()
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
            // 对于不同Component,会有一个一样的静态变量
            // static mut COMPONENT_ID: OnceCell<[TypeId; 1]> = OnceCell::new();

            // 所以用上了哈希表
            static mut COMPONENT_IDS: OnceCell<HashMap<TypeId, [TypeId; 1]>> = OnceCell::new();
            unsafe {
                COMPONENT_IDS.get_or_init(HashMap::new);
                let hashset = COMPONENT_IDS.get_mut().unwrap();
                hashset
                    .entry(Self::type_id_())
                    .or_insert_with(|| [Self::type_id_()]);
                hashset.get(&Self::type_id_()).unwrap()
            }
        }

        fn type_name() -> &'static str {
            type_name::<Self>()
        }

        fn type_id_() -> TypeId {
            TypeId::of::<Self>()
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
                "bundle: {}\nnr_components: {}",
                B::type_name(),
                B::components_ids().len(),
            )
        }

        // 编辑器没报错就算是通过了
        print_bundle_meta::<i32>();
        print_bundle_meta::<(i32, usize)>();
        print_bundle_meta::<(&str, usize)>();
    }
}
