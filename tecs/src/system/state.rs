use std::{
    any::{type_name, Any, TypeId},
    collections::HashMap,
};

use crate::traits::fetch::WorldFetch;

/// [System]的"状态"
///
/// 计算[System]获取的资源是否会破坏别名规则,
/// 资源竞争等
///
/// 算是最后抢救一下unsafe遍布的代码吧
///
/// 仅仅在[System]第一次执行时进行计算,
pub struct SystemState {
    pub(crate) alias_map: AliasMap,
}

impl SystemState {
    pub fn new() -> Self {
        Self {
            alias_map: Default::default(),
        }
    }
}

impl Default for SystemState {
    fn default() -> Self {
        Self::new()
    }
}

/// 用来检测别名冲突
///
/// 枚举引用的使用情况
pub enum Alias {
    /// 有不可变引用
    Imut,
    /// 有可变引用
    Mut,
}

impl Alias {
    /// Returns `true` if the alias is [`Imut`].
    ///
    /// [`Imut`]: Alias::Imut
    #[must_use]
    pub fn is_imut(&self) -> bool {
        matches!(self, Self::Imut)
    }

    /// Returns `true` if the alias is [`Mut`].
    ///
    /// [`Mut`]: Alias::Mut
    #[must_use]
    pub fn is_mut(&self) -> bool {
        matches!(self, Self::Mut)
    }
}

/// 计算别名冲突
pub struct AliasMap {
    /// <类型的ID,(别名情况,使用类型的Fetch)
    inner: HashMap<TypeId, (Alias, Vec<&'static str>)>,
}

impl AliasMap {
    pub fn new() -> Self {
        Self {
            inner: Default::default(),
        }
    }

    pub fn insert<F: WorldFetch, T: Any>(&mut self, usage: Alias) {
        let (ty, ty_name) = (TypeId::of::<T>(), type_name::<T>());
        if self.inner.contains_key(&ty) {
            let (alias, users) = self.inner.get_mut(&ty).unwrap();
            if usage.is_mut() && alias.is_imut() {
                let users = users
                    .iter()
                    .fold(String::new(), |str, user| str + user + " ");
                panic!("发生别名冲突: WorldFetch [{}] 都使用了{}的不可变引用,而 WorldFetch {}使用了{}的可变引用\n",
                    users,ty_name,type_name::<F>(),ty_name);
            }
            if usage.is_imut() && alias.is_mut() {
                panic!("发生别名冲突: WorldFetch [{}] 使用了{}的不可变引用,而 WorldFetch {}使用了{}的可变引用\n",
                    type_name::<F>(),ty_name,users[0],ty_name);
            }
            users.push(type_name::<F>());
        } else {
            self.inner.insert(ty, (usage, vec![type_name::<F>()]));
        }
    }
}

impl Default for AliasMap {
    fn default() -> Self {
        Self::new()
    }
}
