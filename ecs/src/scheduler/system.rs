use super::{fetch::WorldFetch, query::WorldQuery};

/// ECS之[System]
///
/// 主要是对[World]产生效果
///
/// [World]: crate::world::World
pub trait System {
    type Fetch: WorldFetch;
    type Query: WorldQuery;
    fn init(&mut self) -> Vec<Box<dyn SystemState>>;

    fn run(&mut self, states: &mut Vec<Box<dyn SystemState>>);
}
pub trait SystemState {}
