use std::{
    any::{Any, TypeId},
    cell::UnsafeCell,
    collections::HashMap,
};

use crate::{traits::resources::ResManager, world::World};

use super::FnSystemParm;

pub struct Res<'a, T: 'static> {
    handle: &'a mut Option<Box<T>>,
}

impl<'a, T: 'static> Res<'a, T> {
    pub(crate) fn new(res: &mut Option<Box<dyn Any>>) -> Res<'_, T> {
        let handle = unsafe { std::mem::transmute(res) };
        Res { handle }
    }

    /// 获取资源的不可变引用,或者初始化资源
    ///
    /// + 如果原来有资源,会返回资源的不可变引用
    ///
    /// + 如果原来没有资源，会调用init函数然后返回资源的不可变引用
    pub fn get_or_init<F>(&mut self, init: F) -> &T
    where
        F: FnOnce() -> T,
    {
        if self.handle.is_none() {
            *self.handle = Some((init)().into());
        }
        self.get().unwrap()
    }

    /// 获取资源的不可变引用
    pub fn get(&self) -> Option<&T> {
        self.handle.as_ref().and_then(|box_| Some(&**box_))
    }

    /// 获取资源的可变引用
    pub fn get_mut(&mut self) -> Option<&mut T> {
        self.handle.as_mut().and_then(|box_| Some(&mut **box_))
    }

    /// 取得资源
    ///
    /// + 如果原来有资源,会返回[Some]并且移除[World]中的资源
    ///
    /// + 如果原来没有资源，返回[None]
    pub fn take(&mut self) -> Option<Box<T>> {
        self.handle.take()
    }

    /// 删除资源
    ///
    /// 如果原来存在资源,会删除资源
    ///
    /// 否则什么都不做
    pub fn remove(&mut self) {
        *self.handle = None;
    }
}

impl<'a, T: 'static> FnSystemParm for Res<'a, T> {
    unsafe fn build(world: &World) -> Self {
        #[allow(mutable_transmutes)]
        let world: &mut World = std::mem::transmute(world);
        std::mem::transmute(world.get_res::<T>())
    }

    unsafe fn init(state: &mut crate::system::state::SystemState) {
        if state.resources || state.res.contains(&TypeId::of::<T>()) {
            panic!("Res不可和Resources或者重复的Res共存")
        }
        state.res.insert(TypeId::of::<T>());
    }
}

pub struct Resources<'a> {
    resources: &'a mut HashMap<TypeId, UnsafeCell<Option<Box<dyn Any>>>>,
}

impl<'a> ResManager for Resources<'a> {
    fn get_res<T: 'static>(&mut self) -> Res<'_, T> {
        let t_id = TypeId::of::<T>();
        if !self.resources.contains_key(&t_id) {
            self.resources.insert(t_id, UnsafeCell::new(None));
        }
        let res = self.resources.get_mut(&t_id).unwrap().get_mut();
        Res::new(res)
    }

    fn try_get_res<T: 'static>(&mut self) -> Option<Res<'_, T>> {
        let t_id = TypeId::of::<T>();
        let res = self.resources.get_mut(&t_id)?.get_mut();
        Some(Res::new(res))
    }

    fn new_res<T: 'static>(&mut self) {
        let t_id = TypeId::of::<T>();
        if !self.resources.contains_key(&t_id) {
            self.resources.insert(t_id, UnsafeCell::new(None));
        }
    }
}

impl FnSystemParm for Resources<'_> {
    unsafe fn build(world: &World) -> Self {
        #[allow(mutable_transmutes)]
        let world: &mut World = std::mem::transmute(world);
        Self {
            resources: &mut world.resources,
        }
    }

    unsafe fn init(state: &mut crate::system::state::SystemState) {
        // 理论上因为UnsafeCell会自己在运行时painc
        // 但是还是提前制止吧?
        if state.resources || state.res.len() != 0 {
            panic!("Resources不可和其他Resources或者任何Res共存")
        }
        state.resources = true;
    }
}
