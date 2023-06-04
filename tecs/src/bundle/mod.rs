mod component;
mod meta;
use std::any::{Any, TypeId};

pub use component::*;
pub use meta::BundleMeta;

/// 一系列[Component]的组合
///
/// + 任何由[Components]构成的元组
///
/// + 任何由[Components]构成,并且drive了本特征的类型
pub trait Bundle: Any {
    /// 对[Bundle]中的全部[Component]的引用    
    fn destory(self) -> Components;

    /// [Bundle]中所有[Component]的[TypeId]
    fn components_ids(&self) -> &[TypeId];
}
