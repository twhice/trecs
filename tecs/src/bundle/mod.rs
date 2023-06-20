mod component;
mod meta;
use std::any::{Any, TypeId};

pub use component::{Component, Components};
pub(crate) use meta::BundleMeta;
pub use proc::{Bundle, Component};

/// 一系列[Component]的组合
///
/// + 任何由[Components]构成的元组
///
/// + 任何由[Components]构成,并且drive了本特征的类型
pub trait Bundle: Any {
    /// 对[Bundle]中的全部[Component]的引用    
    fn destory(self) -> Components;

    /// [Bundle]中所有[Component]的[TypeId]
    fn components_ids() -> &'static [TypeId];

    /// 还原并[Drop]
    ///
    /// 主要是传递给[BundleMeta]，作为[World][Drop]时的调用
    ///
    /// 否则类型不会正常地[Drop::drop]
    fn drop(cs: Components);

    /// [Bundle]的类型名,是为了方便加上的
    fn type_name() -> &'static str;

    /// [Bundle]的[TypeId],是为了方便加上的
    ///
    /// 多加一个"-"是为了和[Any::type_id()]区分开
    fn type_id_() -> TypeId;
}
