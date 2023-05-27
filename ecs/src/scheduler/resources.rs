use std::{
    any::{Any, TypeId},
    ops::{Deref, DerefMut},
};

use crate::world::traits::InnerResources;

pub struct Resources<'a> {
    inner: &'a mut dyn InnerResources,
}

impl<'a> Resources<'a> {
    pub fn new<R: InnerResources>(s: &'a mut R) -> Self {
        Self { inner: s }
    }

    pub fn get_res<R: Any>(&self) -> Option<&R> {
        self.inner.inner_get_res(TypeId::of::<R>())?.downcast_ref()
    }

    pub fn get_res_mut<R: Any>(&mut self) -> Option<&mut R> {
        self.inner
            .inner_get_res_mut(TypeId::of::<R>())?
            .downcast_mut()
    }

    pub fn contain<R: Any>(&self) -> bool {
        self.inner.inner_contain(TypeId::of::<R>())
    }
}

pub struct Res<'a, T: 'static> {
    pub(crate) inner: &'a mut T,
}

impl<T: 'static> Deref for Res<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.inner
    }
}

impl<T> DerefMut for Res<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.inner
    }
}
