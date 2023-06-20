use std::marker::PhantomData;

#[cfg(feature = "system")]
use crate::system::fnsys::FnSystemParm;
use crate::{
    iter::{EIter, Iter},
    traits::{fetch::WorldFetch, filter::WorldFilter},
    world::World,
};

#[allow(unused_imports)]
use crate::bundle::Components;
/// [FnSystem]的参数之一
///
/// 用来操作从world中选定的部分[Components]
///
/// 有可能会出现别名冲突导致[FnSystem]第一次运行时painc
///
/// [FnSystem]: system::fnsys::FnSystem
#[derive(Clone)]
pub struct Query<'a, F: WorldFetch, Q: WorldFilter = ()> {
    world: &'a World,
    _p: PhantomData<(F, Q)>,
}

impl<'a, F: WorldFetch, Q: WorldFilter> Query<'a, F, Q> {
    pub fn new(world: &'_ World) -> Query<'_, F, Q> {
        Query {
            world,
            _p: PhantomData,
        }
    }

    pub fn into_eiter(self) -> EIter<'a, F> {
        unsafe {
            #[allow(mutable_transmutes)]
            EIter::new::<Q>(std::mem::transmute(self.world))
        }
    }
}

impl<'a, F: WorldFetch + 'a, Q: WorldFilter> IntoIterator for Query<'a, F, Q> {
    type Item = F::Item<'a>;

    type IntoIter = Iter<'a, F>;

    fn into_iter(self) -> Self::IntoIter {
        unsafe {
            #[allow(mutable_transmutes)]
            Iter::new::<Q>(std::mem::transmute(self.world))
        }
    }
}

#[cfg(feature = "system")]
impl<F: WorldFetch, Q: WorldFilter> FnSystemParm for Query<'_, F, Q> {
    unsafe fn build(world: &World) -> Self {
        let world: &'static World = std::mem::transmute(world);
        Query::<'_, F, Q>::new(world)
    }

    unsafe fn init(state: &mut crate::system::state::SystemState) {
        F::alias_conflict(&mut state.alias_map);
    }
}
