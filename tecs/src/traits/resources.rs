use crate::system::fnsys::Res;
#[allow(unused_imports)]
use crate::world::World;

pub trait ResManager {
    /// 获取类型对应资源的一个[ResOwner]
    ///
    /// 重复获取会报错
    ///
    /// 如果原来不存在资源,会为资源创建一个位置
    fn get_res<T: 'static>(&mut self) -> Res<'_, T>;

    /// 获取类型对应资源的一个[ResOwner]
    ///
    /// 如果原来不存在资源,不会为资源创建一个位置,并返回[none]
    fn try_get_res<T: 'static>(&mut self) -> Option<Res<'_, T>>;

    /// 为类型创建一个位置,准备储存资源
    ///
    /// 如果原来的资源存在,什么都不做
    fn new_res<T: 'static>(&mut self);
}
