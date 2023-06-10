mod commands;

mod query;
mod resources;
pub use self::{
    commands::Commands,
    query::Query,
    resources::{Res, Resources},
};

use crate::world::World;

use super::{state::SystemState, System};

/// 函数系统 : 由实现了[FnSystemParm]特征的类型作为参数,并且加上
/// [proc::system]属性的的函数
pub trait FnSystem: 'static {
    unsafe fn fn_run_once(&mut self, world: &World);
}

/// 实现此特征 就可以作为[FnSystem]的参数
pub trait FnSystemParm {
    /// 从[World]创建
    unsafe fn build(world: &World) -> Self;

    /// 初始化,通过[SystemState]保证安全性
    unsafe fn init(state: &mut SystemState);
}

impl<F> FnSystem for F
where
    F: FnMut(&World) + 'static,
{
    unsafe fn fn_run_once(&mut self, world: &World) {
        (self)(world);
    }
}

impl<S: FnSystem> System for S {
    unsafe fn run_once(&mut self, world: &World) {
        self.fn_run_once(world);
    }
}
