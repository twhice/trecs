use crate::{traits::command::Command, World};

use super::FnSystemParm;

pub struct Commands<'a> {
    inner: &'a mut World,
}

impl Commands<'_> {
    pub(crate) fn new(world: &mut World) -> Commands<'_> {
        Commands { inner: world }
    }
}

impl Command for Commands<'_> {
    fn register<B: crate::bundle::Bundle>(&mut self) {
        self.inner.register::<B>()
    }

    fn spawn<B: crate::bundle::Bundle>(&mut self, b: B) -> crate::storage::Entity {
        self.inner.spawn(b)
    }

    fn spawn_many<B: crate::bundle::Bundle, I: IntoIterator<Item = B>>(
        &mut self,
        i: I,
    ) -> Vec<crate::storage::Entity> {
        self.inner.spawn_many(i)
    }

    fn alive(&self, entity: crate::storage::Entity) -> Option<bool> {
        self.inner.alive(entity)
    }

    fn remove(&mut self, entity: crate::storage::Entity) -> bool {
        self.inner.remove(entity)
    }
}

impl FnSystemParm for Commands<'_> {
    unsafe fn build(world: &World) -> Self {
        #[allow(mutable_transmutes)]
        let world: &mut World = std::mem::transmute(world);
        Self::new(world)
    }

    unsafe fn init(_state: &mut crate::system::state::SystemState) {
        // commands可以重复
    }
}
