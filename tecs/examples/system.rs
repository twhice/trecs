use std::time::{Duration, Instant};

use tecs::proc::fnsystem;
use tecs::system::fnsys::query::Query;
use tecs::traits::command::Command;
use tecs::world::World;

fn main() {
    // 直接输出
    #[fnsystem]
    fn hello_world() {
        println!("Hello World 1")
    }

    // 从世界中选取字符串,然后输出
    #[fnsystem] // Query<'a,F,Q> 这里F是&'static &str代表获取全部&str的不可变引用
    fn hello_world_from_cs(query: Query<&'static &str>) {
        for str in query {
            println!("{str}")
        }
    }

    // 计时器 五秒后开始返回true
    let start = Instant::now();
    let delay = || start.elapsed() > Duration::from_secs(5);

    // 创建世界,创建两个entity
    let mut world = World::new();
    world.spawn("hello world 2");
    world.spawn("hello world 3");

    // 应该看到交替的HelloWorld 1 2 3
    // 持续5秒
    world
        .add_system(hello_world)
        .add_system(hello_world_from_cs)
        .run_until(delay);
}
