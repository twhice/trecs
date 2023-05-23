use crate::world::World;

pub trait System
where
    World: AsMut<Self>,
{
}

impl<T> System for T where World: AsMut<T> {}
