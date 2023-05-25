pub(crate) mod bundle;
pub(crate) mod entity;
use std::any::Any;

pub trait Component: Any {
    const COMPONENT_FLAG: () = ();
}

mod _impl {
    macro_rules! impl_components {
        ($($t:ty),*) => {
            #[rustfmt::skip]
            $(impl super::Component for $t{})*
        };
    }

    impl_components!(usize, u128, u64, u32, u16, u8);
    impl_components!(isize, i128, i64, i32, i16, i8);
    impl_components!(bool, char, (), &'static str);
}
