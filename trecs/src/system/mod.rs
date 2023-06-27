pub(crate) mod state;
use crate::world::World;
use state::SystemState;

/// 函数系统 : 由实现了[FnSystemParm]特征的类型作为参数,并且加上
/// [proc::system]属性的的函数
pub trait InnerSystem<Marker> {
    /// 从[World]创建参数
    fn build_args(&self, world: &World) -> Box<()>;

    /// 初始化
    fn init(&self);

    fn run_once(&mut self, args: Box<()>);
}

/// 实现此特征 就可以作为[System]的参数
pub(crate) trait SystemParm {
    /// 从[World]创建
    ///
    /// # Safety
    ///
    /// 这个函数的安全性通过[FnSystemParm::init]保证
    unsafe fn build(world: &World) -> Self;

    /// 初始化,通过[SystemState]保证安全性
    fn init(state: &mut SystemState);
}

mod __impl {
    use super::{InnerSystem, SystemParm, SystemState, World};

    macro_rules! impl_fnsystem {
        ($($t:ident),*) => {
            impl<F,$($t : SystemParm,)*> InnerSystem<($($t,)*)> for F
            where F : FnMut($($t,)*) {
                fn build_args(&self, world: &World) -> Box<()>{
                    unsafe{
                        std::mem::transmute(Box::new(($($t::build(world),)*)))
                    }
                }

                fn init(&self) {
                    let mut state = SystemState::new();
                    $($t::init(&mut state);)*
                }

                fn run_once(&mut self, args: Box<()>) {
                    let ($($t,)*) = unsafe{
                        *std::mem::transmute::<_,Box<($($t,)*)>>(args)
                    };
                    (self)($($t,)*)
                }
            }
        };
    }
    trecs_proc::all_tuple!(impl_fnsystem, 16);

    impl<F> InnerSystem<()> for F
    where
        F: FnMut(),
    {
        fn build_args(&self, _world: &World) -> Box<()> {
            Box::new(())
        }

        fn init(&self) {}

        fn run_once(&mut self, _args: Box<()>) {
            (self)()
        }
    }
}

pub struct System {
    inner: Box<dyn InnerSystem<()>>,
}

impl System {
    pub(crate) fn new<M, F: InnerSystem<M>>(fn_system: F) -> Self {
        fn_system.init();
        let fn_system: Box<dyn InnerSystem<M>> = Box::new(fn_system);
        Self {
            inner: unsafe { std::mem::transmute(fn_system) },
        }
    }

    pub(crate) fn run_once(&mut self, world: &World) {
        self.inner.run_once(self.inner.build_args(world));
    }
}
