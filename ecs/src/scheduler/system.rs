use std::{
    any::{type_name, TypeId},
    collections::BTreeSet,
    process::exit,
};

use super::{
    access::Access,
    command::Command,
    fetch::WorldFetch,
    query::WorldQuery,
    resources::{Res, Resources},
    transmute_lifetime, Scheduler,
};

pub struct SystemState {
    res: BTreeSet<TypeId>,
    resources: bool,
    access: bool,
    command: bool,
}

impl SystemState {
    pub fn new() -> Self {
        Self {
            res: Default::default(),
            access: false,
            command: false,
            resources: false,
        }
    }

    pub unsafe fn new_res<T: 'static>(&self) {
        let this = unsafe { transmute_lifetime(self) };
        if this.res.insert(TypeId::of::<T>()) {
            eprintln!("一个System中不可以有重复的Res<{}>", type_name::<T>());
            exit(-1);
        } else if this.resources {
            eprintln!("一个System中只可以有一个Resources或者多个Res");
            exit(-1);
        }
    }

    pub unsafe fn new_resources(&self) {
        let this = unsafe { transmute_lifetime(self) };
        if this.res.len() != 0 {
            eprintln!("一个System中只可以有一个Resources或者多个Res");
            exit(-1);
        } else if this.resources {
            eprintln!("一个System中只可以有至多一个Resources");
            exit(-1);
        }
        this.resources = true;
    }

    pub unsafe fn new_access(&self) {
        let this = transmute_lifetime(self);

        if this.access {
            eprintln!("一个System中至多有一个Access");
            exit(-1);
        } else {
            this.access = true;
        }
    }

    pub unsafe fn new_command(&self) {
        let this = transmute_lifetime(self);
        if this.command {
            eprintln!("一个System中至多有一个Command");
            exit(-1);
        } else {
            this.command = true;
        }
    }
}

pub trait Module<'a> {
    fn init(scheduler: &mut Scheduler);
    fn create(scheduler: &'a Scheduler) -> Self;
}

impl<'a, F: WorldFetch, Q: WorldQuery> Module<'a> for Access<'a, F, Q> {
    fn init(scheduler: &mut Scheduler) {
        unsafe { scheduler.temp_system_state.new_access() };
        scheduler.registry_fetch::<F>();
        scheduler.registry_query::<Q>();
    }

    fn create(scheduler: &'a Scheduler) -> Self {
        scheduler.new_access()
    }
}

impl<'a, T: 'static> Module<'a> for Res<'a, T> {
    fn init(scheduler: &mut Scheduler) {
        unsafe { scheduler.temp_system_state.new_res::<T>() }
    }

    fn create(scheduler: &'a Scheduler) -> Self {
        let scheduler = unsafe { transmute_lifetime(scheduler) };
        let inner = scheduler
            .world
            .resources
            .get_mut(&TypeId::of::<T>())
            .expect(&format!("不存在的Resources: \"{}\"", type_name::<T>()))
            .downcast_mut()
            .unwrap();

        Res { inner }
    }
}

impl<'a> Module<'a> for Resources<'a> {
    fn init(scheduler: &mut Scheduler) {
        unsafe { scheduler.temp_system_state.new_resources() }
    }

    fn create(scheduler: &'a Scheduler) -> Self {
        scheduler.new_resources()
    }
}

impl<'a> Module<'a> for Command<'a> {
    fn init(scheduler: &mut Scheduler) {
        unsafe {
            scheduler.temp_system_state.new_command();
        }
    }

    fn create(scheduler: &'a Scheduler) -> Self {
        scheduler.new_command()
    }
}

impl Default for SystemState {
    fn default() -> Self {
        Self::new()
    }
}

pub trait System<'a> {
    /// 初始化
    ///
    /// 比如初始化缓存信息
    fn init(&mut self, scheduler: &mut Scheduler);
    /// 运行一次
    fn run_once(&mut self, scheduler: &'a Scheduler);
}

mod _impl {
    use super::*;
    macro_rules! system {
        ($($m:ident),*) => {
            #[rustfmt::skip]
            impl <'a,$($m: Module<'a>,)*> System<'a> for fn($($m,)*){
                fn init(&mut self, scheduler: &mut Scheduler) {
                    $($m::init(scheduler);)*
                }

                fn run_once(&mut self, scheduler: &'a Scheduler) {
                    (self)($($m::create(scheduler)),*)
                }
            }
        };
    }

    system!(M1);
    system!(M1, M2);
    system!(M1, M2, M3);
    system!(M1, M2, M3, M4);
    // ...
}
