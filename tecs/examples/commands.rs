use std::thread;
use std::time::{Duration, Instant};

use proc::Component;
use tecs::proc::fnsystem;
use tecs::system::fnsys::Query;
use tecs::World;
use tecs::{system::fnsys::Commands, traits::command::Command};

#[derive(Component)]
pub struct Str {
    // 因为String并不是一个Component
    inner: String,
}

// unsafe会被忽视,最终的函数是safe的
// 但是代码块仍然是unsafe上下文
#[fnsystem]
unsafe fn spawn_hello_world(mut commands: Commands) {
    static mut COUNTER: usize = 0;

    commands.spawn(Str {
        inner: String::from("hello world ") + &COUNTER.to_string(),
    });
    COUNTER += 1;
}

#[fnsystem]
fn print_hello_world(q: Query<&Str>, mut commands: Commands) {
    for eb in q.into_eiter() {
        // 读取,然后删除
        println!("{}", eb.inner);
        commands.remove(eb.entity());
    }
}

#[fnsystem]
fn twice_pre_s() {
    thread::sleep(Duration::from_secs_f64(1.0 / 2.0))
}

fn main() {
    let mut world = World::new();
    let start = Instant::now();
    world
        .add_system(spawn_hello_world)
        .add_system(print_hello_world)
        // 两秒后结束
        .add_system(twice_pre_s)
        .run_until(|| start.elapsed() > Duration::from_secs(2));
}
