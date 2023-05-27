use std::any::{Any, TypeId};

use crate::component::entity::Entity;

use super::chunk::Components;

/// 对[World]产生操作
///
/// 诸如
///
/// * 插入[Components]
///
/// * 插入大量[Components]
///
/// * 删除[Components]
///
/// * 删除并且返回[Components]
///
/// * 查看[Entity]是否有效
///
/// [world]: crate
pub trait InnerCommand {
    /// 删除[Entity]对应的[Components]
    ///
    /// * 如果[Entity]存在 返回true
    ///
    /// * 否则,返回false并且什么都不做
    fn remove(&mut self, entity: Entity) -> bool;
    /// 删除[Entity]对应的[Components],并且返回[Components]
    ///
    /// * 如果[Entity]存在 [Some(Components)]
    ///
    /// * 否则,返回[None]
    ///
    /// 相比于remove,这个方法的开销更大
    fn r#return(&mut self, entity: Entity) -> Option<Components>;
    /// 获取[Entity]是否存在,是否有效
    ///
    /// * [Entity]不存在时返回[None]
    ///
    /// * 返回[Some(bool)]时,[bool]表示[Entity]是否有效
    fn alive(&self, entity: Entity) -> Option<bool>;

    /// 注册一个[Bundle],生成[BundleMeta]
    ///
    /// * 如果[Bundle]已经存在,什么都不做
    ///
    /// [Bundle]: crate
    /// [BundleMeta]: crate
    fn inner_register(&mut self, bundle_id: TypeId, components: &'static [TypeId]) -> bool;

    /// 放入一个[Components],返回[Entity]
    ///
    /// 通过有效的[Entity]可以对[Components]操作
    fn inner_spawn(
        &mut self,
        bundle_id: TypeId,
        components: &'static [TypeId],
        bundle: Components,
    ) -> Entity;

    /// 一次性插入大量同类[Components]
    ///
    /// 相比逐个sapwn,具有更高的性能  
    fn inner_spawn_many(
        &mut self,
        bundle_id: TypeId,
        components: &'static [TypeId],
        bundles: Vec<Components>,
    ) -> Vec<Entity>;
}

/// object-safe需要 没有泛型很烦
pub trait InnerResources {
    fn inner_get_res_mut(&mut self, resources_id: TypeId) -> Option<&mut Box<dyn Any>>;
    fn inner_get_res(&self, resources_id: TypeId) -> Option<&Box<dyn Any>>;
    fn inner_contain(&self, resources_id: TypeId) -> bool;
}
