use std::any::TypeId;

use crate::component::{bundle::Bundle, entity::Entity};

use crate::world::{chunk::Components, traits::InnerCommand};

pub struct Command<'a> {
    inner: &'a mut dyn InnerCommand,
}

impl<'a> Command<'a> {
    pub fn new<C: InnerCommand>(s: &'a mut C) -> Self {
        Self { inner: s }
    }

    pub fn register<B: Bundle>(&mut self) -> bool {
        self.inner
            .inner_register(TypeId::of::<B>(), B::conponents_ids())
    }

    pub fn spawn<B: Bundle>(&mut self, b: B) -> Entity {
        self.inner
            .inner_spawn(TypeId::of::<B>(), B::conponents_ids(), b.deconstruct())
    }

    pub fn spawn_many<B: Bundle, I: Iterator<Item = B>>(&mut self, bs: I) -> Vec<Entity> {
        let bundles = bs.map(|b| b.deconstruct());
        self.inner
            .inner_spawn_many(TypeId::of::<B>(), B::conponents_ids(), bundles.collect())
    }

    pub fn alive(&self, entity: Entity) -> Option<bool> {
        self.inner.alive(entity)
    }

    pub fn remove(&mut self, entity: Entity) -> bool {
        self.inner.remove(entity)
    }

    pub fn r#return(&mut self, entity: Entity) -> Option<Components> {
        self.inner.r#return(entity)
    }
}
