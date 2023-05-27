use std::any::{Any, TypeId};
use std::marker::PhantomData;

use crate::component::bundle::Bundle;

#[allow(unused_imports)]
use crate::world::World;
/// 检查[Bundle]是否满足特定要求
///
/// 最为[WorldFetch]的附属进一步筛选[Bundle]
///
/// [WorldFetch]: crate
pub trait WorldQuery: Any {
    /// 检查[Bundle]是否满足特定要求
    fn pass(components: &[TypeId]) -> bool;
}

/// [Bundle]必须包含其中全部
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

/// [Bundle]不能包含其中任意一个
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

/// [Bundle]需要包含其中任意一个
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

/// 特例,表示没有筛选
impl WorldQuery for () {
    fn pass(_components: &[TypeId]) -> bool {
        true
    }
}
