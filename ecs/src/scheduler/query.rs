use std::any::{Any, TypeId};
use std::marker::PhantomData;

use crate::component::bundle::Bundle;

#[allow(unused_imports)]
use crate::world::World;
/// 查询[World]中的元素
///
/// 仅仅查询是否满足要求
pub trait WorldQuery: Any {
    /// 传入
    fn pass(components: &[TypeId]) -> bool;
}

pub struct With<B: Bundle>(PhantomData<B>);

impl<B: Bundle> WorldQuery for With<B> {
    fn pass(components: &[TypeId]) -> bool {
        'main: for wanna in B::conponents_ids() {
            for cid in components {
                if *wanna == *cid {
                    continue 'main;
                }
            }
            return false;
        }
        true
    }
}
pub struct WithOut<B: Bundle>(PhantomData<B>);

impl<B: Bundle> WorldQuery for WithOut<B> {
    fn pass(components: &[TypeId]) -> bool {
        for wanna in B::conponents_ids() {
            for cid in components {
                if *wanna == *cid {
                    return false;
                }
            }
        }
        true
    }
}

pub struct AnyOf<B: Bundle>(PhantomData<B>);

impl<B: Bundle> WorldQuery for AnyOf<B> {
    fn pass(components: &[TypeId]) -> bool {
        for wanna in B::conponents_ids() {
            for cid in components {
                if *wanna == *cid {
                    return true;
                }
            }
        }
        false
    }
}

impl WorldQuery for () {
    fn pass(_components: &[TypeId]) -> bool {
        true
    }
}
