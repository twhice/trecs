use std::time::{Duration, Instant};

use tecs::proc::fnsystem;
use tecs::traits::resources::ResManager;
use tecs::world::{Res, Resources};
use tecs::World;

/// 初始化资源String 为 "Hello world from Res"
#[fnsystem]
fn init_hello_world(mut res: Res<String>) {
    res.get_or_init(|| String::from("Hello world from Res"));
}

/// 通过Res直接访问资源
#[fnsystem]
fn print_hello_world1(res: Res<String>) {
    println!("{}", res.get().unwrap());
}

/// 通过Resources访问资源
#[fnsystem]
fn print_hello_world2(mut res: Resources) {
    let res = res.get_res::<String>();
    let hw = res.get().unwrap();
    println!("`{}` from Resources", hw)
}

fn main() {
    let mut world = World::new();
    let start = Instant::now();
    world
        .add_startup_system(init_hello_world)
        .add_system(print_hello_world1)
        .add_system(print_hello_world2)
        // 两秒后结束
        .run_until(|| start.elapsed() > Duration::from_secs(2));
}
