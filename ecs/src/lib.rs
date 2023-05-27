pub(crate) mod component;
pub(crate) mod scheduler;
pub(crate) mod world;

pub use scheduler::{Schedule, Scheduler};
pub use world::World;

pub mod iter {
    pub use crate::scheduler::iter::*;
}
pub mod modules {
    pub use crate::scheduler::access::Access;
    pub use crate::scheduler::command::Command;
    pub use crate::scheduler::resources::*;
}

#[cfg(test)]
mod tests {

    fn greet() {
        println!("欢迎使用异月写的垃圾ecs库!")
    }

    fn spawn(mut command: Command) {
        command.spawn("Hello World");
    }

    fn print(mut access: Access<&'static &str>) {
        for str in access.bundle_iter() {
            println!("{}", str);
        }
    }

    use modules::{Access, Command};

    use crate::scheduler::system::System;

    use super::*;

    #[test]
    fn test_name() {
        impl<'a> System<'a> for fn() {
            fn init(&mut self, _scheduler: &mut Scheduler) {}

            fn run_once(&mut self, _scheduler: &'a Scheduler) {
                (self)()
            }
        }
        let mut world = World::new();
        // 什么鸡巴
        // 为什么不行
        let mut schedule = world.scheduler().add_system(greet).build();

        schedule.run();
    }
}
