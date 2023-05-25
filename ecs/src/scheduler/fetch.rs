use std::any::TypeId;

use crate::{component::Component, world::chunk::Components};

use super::transmute_lifetime;

/// 映射表
///
/// 一种树状的数据结构
///
/// 用于嵌套地通过[Component]生成WorldFetch::Item
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MappingTable {
    Node(Vec<MappingTable>),
    Mapping(usize),
}

impl MappingTable {
    pub fn new<F>(size: usize, mut fill_with: F) -> Self
    where
        F: FnMut() -> Self,
    {
        let mut node = Vec::with_capacity(size);
        for _ in 0..size {
            node.push((fill_with)())
        }
        Self::Node(node)
    }

    pub fn as_mapping(&self) -> Option<&usize> {
        if let Self::Mapping(v) = self {
            Some(v)
        } else {
            None
        }
    }

    pub fn as_node(&self) -> Option<&Vec<MappingTable>> {
        if let Self::Node(v) = self {
            Some(v)
        } else {
            None
        }
    }
}

impl From<Vec<usize>> for MappingTable {
    fn from(value: Vec<usize>) -> Self {
        let size = value.len();
        let mut iter = value.into_iter();

        Self::new(size, || Self::Mapping(iter.next().unwrap()))
    }
}

/// 一种"索引"
///
/// 能够在[World]中选取出特定的元素
///
/// 通过创建[MappingTable]并根据这个[MappingTable]来实现
///
/// [World]: crate::world::World
pub trait WorldFetch {
    type Item<'a>;

    /// 通过[Components]生成Item
    ///
    /// 内部使用了一些rust unsafe黑魔法
    ///
    /// 不会检查[MappingTable]是否有效,是否正确
    unsafe fn build<'a>(components: &Components, mapping: &MappingTable) -> Self::Item<'a>;

    /// 用来检测能不能从[Components]中取得Item
    ///
    /// 如果能 会返回一个[MappingTable]
    ///
    /// 调用时应该新建一个[Vec]并传入一个可变引用
    ///
    /// 函数签名如此是为了方便递归
    fn contain(components: &mut Vec<TypeId>) -> Option<MappingTable>;
}

impl<T: Component> WorldFetch for &T {
    type Item<'a> = &'a T;

    #[inline]
    unsafe fn build<'a>(components: &Components, mapping: &MappingTable) -> Self::Item<'a> {
        let mapping = *mapping.as_mapping().unwrap();
        transmute_lifetime(components[mapping].downcast_ref().unwrap())
    }

    /// 最简单的情况 : 只需要获取一个引用
    fn contain(components: &mut Vec<TypeId>) -> Option<MappingTable> {
        let mut index = None;
        let self_id = TypeId::of::<T>();

        // 遍历查看 是否包含
        for (idx, tid) in components.iter().enumerate() {
            if *tid == self_id {
                index = Some(idx);
                break;
            }
        }
        let index = index?;
        // 避免别名 移除
        components.remove(index);
        Some(MappingTable::Mapping(index))
    }
}

impl<T: Component> WorldFetch for &mut T {
    type Item<'a> = &'a mut T;

    #[inline]
    unsafe fn build<'a>(components: &Components, mapping: &MappingTable) -> Self::Item<'a> {
        let mapping = *mapping.as_mapping().unwrap();
        transmute_lifetime(components[mapping].downcast_ref().unwrap())
    }

    fn contain(components: &mut Vec<TypeId>) -> Option<MappingTable> {
        let mut index = None;
        let self_id = TypeId::of::<T>();
        for (idx, tid) in components.iter().enumerate() {
            if *tid == self_id {
                index = Some(idx);
                break;
            }
        }
        let index = index?;
        components.remove(index);
        Some(MappingTable::Mapping(index))
    }
}

mod _impl {
    use super::*;
    macro_rules! impl_fetch {
        ($($t:ident),*) => {
            #[rustfmt::skip]
            impl<$($t:WorldFetch),*> WorldFetch for ($($t,)*) {
                type Item<'a> = ($($t::Item<'a>,)*);

                #[inline]
                unsafe fn build<'a>(components: &Components, mapping: &MappingTable) -> Self::Item<'a> {
                    let mut nodes = mapping.as_node().unwrap().iter();
                    (
                        $($t::build(components, nodes.next().unwrap()),)*
                    )
                }

                fn contain(components: &mut Vec<TypeId>) -> Option<MappingTable> {
                    let nodes = vec![
                        $($t::contain(components)?),*
                    ];
                    Some(MappingTable::Node(nodes))
                }
            }
        };
    }

    impl_fetch!(A);
    impl_fetch!(A, B);
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::any::Any;
    #[test]

    fn feature() {
        #[inline]
        fn into_box_any<T: Any>(v: T) -> (Box<dyn Any>, TypeId) {
            (Box::new(v), TypeId::of::<T>())
        }

        let mut cs = vec![];
        let mut ids = vec![];
        [
            into_box_any(1),
            into_box_any(""), //
            into_box_any(123.0),
        ]
        .into_iter()
        .map(|(c, id)| {
            cs.push(c);
            ids.push(id);
        })
        .count();

        // 读
        {
            let mut ids = ids.clone();
            let table = <(&&str, &i32) as WorldFetch>::contain(&mut ids).unwrap();
            assert_eq!(
                table,
                MappingTable::Node(vec![MappingTable::Mapping(1), MappingTable::Mapping(0)])
            );
            let q = unsafe { <(&&str, &i32) as WorldFetch>::build(&cs, &table) };
            assert_eq!(q, (&"", &1))
        }

        // 写
        {
            let mut ids = ids.clone();
            let table = <(&&str, &mut i32) as WorldFetch>::contain(&mut ids).unwrap();
            assert_eq!(
                table,
                MappingTable::Node(vec![MappingTable::Mapping(1), MappingTable::Mapping(0)])
            );
            let q = unsafe { <(&&str, &mut i32) as WorldFetch>::build(&cs, &table) };
            *q.1 = 100;
            assert_eq!(q, (&"", &mut 100))
        }
    }
}
