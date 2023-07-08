#[allow(unused)]
use crate::{
    bundle::{Bundle, BundleMeta},
    storage::Entity,
    World,
};

use super::WorldFetch;

pub trait Command {
    /// 注册一个[Bundle]
    ///
    /// 为[Bundle]生成[BundleMeta]
    ///
    /// 如果[BundleMeta]已经存在,就什么都不做
    fn register<B: Bundle>(&mut self);
    /// 将[Bundle]放入[World]
    ///
    /// 返回代表被放入[World]的[Bundle]的[Entity]
    fn spawn<B: Bundle>(&mut self, b: B) -> Entity;
    /// 一次性将大量同类型[Bundle]放入[World]
    ///
    /// 返回代表被放入[World]的[Bundle]的[Entity]
    fn spawn_many<B: Bundle, I: IntoIterator<Item = B>>(&mut self, i: I) -> Vec<Entity>;
    /// 检查[Entity]的有效性
    ///
    /// + 返回[None]表示[Entity]指向的[Bundle]不存在
    /// + 返回[Some(true)]表示[Entity]有限并存在
    /// + 返回[Some(false)]表示[Entity]有限但是已经过时
    fn alive(&self, entity: Entity) -> Option<bool>;
    /// 从[World]中删除[Entity]代表的[Bundle]
    ///
    /// 返回[Entity]代表的[Bundle]是否存在
    fn remove(&mut self, entity: Entity) -> bool;
    /// 在[Entity]对应的[Bundle]上进行[WorldFetch]
    fn fetch<F: WorldFetch>(&mut self, entity: Entity) -> Option<F::Item<'_>>;
}
