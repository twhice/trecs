pub(crate) mod fetch;
pub(crate) mod query;
pub(crate) mod system;

/// 常用的无界生命周期转换
///
/// 封装为一个函数
///
/// 极度unsafe 安全性没有保证
#[inline]
pub unsafe fn transmute_lifetime<'a, 'b, T>(x: &'a T) -> &'b mut T {
    &mut *(x as *const _ as *mut T)
}

/// 引用的类型强制转换
///
/// 极度unsafe
#[inline]
pub unsafe fn transmute_ref<T, Y>(x: &T) -> &mut Y {
    &mut *(x as *const _ as *mut Y)
}
