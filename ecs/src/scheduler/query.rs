use std::any::TypeId;
use std::marker::PhantomData;

use crate::component::bundle::Bundle;

#[allow(unused_imports)]
use crate::world::World;
/// 查询[World]中的元素
///
/// 仅仅查询是否满足要求
pub trait WorldQuery {
    /// 传入
    fn pass(components: &[TypeId]) -> bool;
}

pub struct With<B: Bundle>(PhantomData<B>);
pub struct WithOut<B: Bundle>(PhantomData<B>);

pub struct OneOf<B: Bundle>(PhantomData<B>);
