/// 定义[Bundle]相关
///
/// 如[BundleMeta]等
pub mod bundle;
/// [Components]迭代器
///
/// [Components]: crate
pub mod iter;
/// 存储[World]中数据的容器
pub mod storage;

#[cfg(feature = "system")]
pub mod system;
///一些用于操作的trait,以及封装其中的[Command]
///
/// + 对[World]进行[Entity]级操作的[Command]
///
/// [Command]:crate
/// [World]:crate
pub mod tools;
pub mod world;
/// 最终的容器
pub use world::World;
