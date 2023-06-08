pub mod fnsys;
pub mod state;

use crate::world::World;

#[allow(unused_imports)]
use self::fnsys::FnSystem;
/// ecs之系统
///
/// 能够操作[World]的资源和,Componnets
///
/// 目前仅仅支持[FnSystem]
pub trait System: 'static {
    unsafe fn run_once(&mut self, world: &World);
}
