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

pub mod system;
///一些用于操作的trait,以及封装其中的[Command]
///
/// + 对[World]进行[Entity]级操作的[Command]
///
/// [Command]:crate
/// [World]:crate
pub mod traits;
/// 最终的容器
pub mod world;

pub mod proc {
    pub use proc::fnsystem;
}
