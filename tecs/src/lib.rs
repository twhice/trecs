/// 定义[Bundle]相关
///
/// 如[BundleMeta]等
pub mod bundle;
/// 存储[World]中数据的容器
pub mod storage;
///一些用于操作的trait,以及封装其中的[Command]
///
/// + 对[World]进行[Entity]级操作的[InnerCommand]
pub mod traits;
pub mod world;
