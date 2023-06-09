use std::time::{Duration, Instant};

use tecs::bundle::{Bundle, Component};
use tecs::proc::fnsystem;
use tecs::system::fnsys::Query;
use tecs::traits::command::Command;
use tecs::World;

#[derive(Bundle)]
struct Str {
    inner: &'static str,
}

#[derive(Component)]
struct Str2 {
    #[allow(unused)]
    inner: String,
}

fn main() {
    // 直接输出
    #[fnsystem]
    fn hello_world() {
        println!("Hello World from system `hello_world`")
    }

    // 从世界中选取字符串,然后输出
    #[fnsystem] // Query<'a,F,Q> 这里F是&'static &str代表获取全部&str的不可变引用
    fn hello_world_from_cs(query: Query<&'static &str>) {
        for str in query {
            println!("{str}")
        }
    }

    // 类似于锁帧
    #[fnsystem]
    fn wait() {
        let instant = Instant::now();
        while instant.elapsed() < Duration::from_secs_f64(1.0 / 2.0) {}
    }

    // 计时器 五秒后开始返回true
    let start = Instant::now();
    let delay = || start.elapsed() > Duration::from_secs(5);

    // 创建世界,创建几个entity
    let mut world = World::new();

    world.spawn("hello world &'static str");

    // 因为这个也包含&str ,所以这个也会被计算在内
    world.spawn(Str {
        inner: "hello world from &'static str in Str",
    });

    // 不会被任何system选中
    world.spawn(123);

    // 这个包含String但是不包含&str,所以不会显示
    world.spawn(Str2 {
        inner: "u wont see `hello world from String in Str2`".into(),
    });

    // 应该看到交替的HelloWorld
    // 持续5秒,每秒输出两次
    world
        .add_system(hello_world)
        .add_system(hello_world_from_cs)
        .add_system(wait)
        .run_until(delay);
}
