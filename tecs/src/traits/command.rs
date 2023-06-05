use crate::{bundle::Bundle, storage::Entity};

pub trait Command {
    fn register<B: Bundle>(&mut self);
    fn spawn<B: Bundle>(&mut self, b: B) -> Entity;
    fn spawn_many<B: Bundle, I: IntoIterator<Item = B>>(&mut self, i: I) -> Vec<Entity>;
    fn alive(&self, entity: Entity) -> Option<bool>;
    fn remove(&mut self, entity: Entity) -> bool;
}
