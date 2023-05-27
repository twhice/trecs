use std::{any::TypeId, collections::BTreeMap};

use super::{
    fetch::{MappingTable, WorldFetch},
    query::WorldQuery,
};

pub struct FetchCache {
    pub(crate) contain: fn(&mut Vec<TypeId>) -> Option<MappingTable>,
    pub(crate) tables: BTreeMap<TypeId, Option<MappingTable>>,
}

impl FetchCache {
    pub fn new<F: WorldFetch>() -> Self {
        Self {
            contain: F::contain,
            tables: Default::default(),
        }
    }
}

pub struct QueryCache {
    pub(crate) pass: fn(&'static [TypeId]) -> bool,
    pub(crate) tables: BTreeMap<TypeId, bool>,
}

impl QueryCache {
    pub fn new<F: WorldQuery>() -> Self {
        Self {
            pass: F::pass,
            tables: Default::default(),
        }
    }
}
