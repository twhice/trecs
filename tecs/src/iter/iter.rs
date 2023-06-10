use std::marker::PhantomData;

use crate::{
    storage::{Chunk, ChunkIter},
    traits::{
        fetch::{MappingTable, WorldFetch},
        filter::WorldFilter,
    },
    world::World,
};

#[derive(Debug, Clone)]
pub(crate) struct Select<'a> {
    inner: Vec<(&'a MappingTable, Vec<&'a Chunk>)>,
}

impl Select<'_> {
    pub fn new<F: WorldFetch, Q: WorldFilter>(world: &'_ mut World) -> Select<'_> {
        world
            .metas
            .iter_mut()
            .filter_map(|(.., meta)| {
                if meta.filter::<Q>() && meta.fetch::<F>().is_some() {
                    Some(meta)
                } else {
                    None
                }
            })
            .filter_map(|meta| {
                let chunks = meta
                    .chunks
                    .iter()
                    .copied()
                    .map(|cid| &world.chunks[cid])
                    .collect::<Vec<_>>();
                let mapping_table = meta.fetch::<F>()?;
                Some((chunks, mapping_table))
            })
            .map(|(a, b)| (b, a))
            .collect::<Vec<_>>()
            .into()
    }

    pub fn pop(&mut self) -> Option<(&'_ MappingTable, &'_ Chunk)> {
        let (mapping, chunks) = self.inner.last_mut()?;
        let Some(chunk) = chunks.pop() else{
            self.inner.pop();
            return self.pop();
        };
        Some((mapping, chunk))
    }
}

impl<'a> From<Vec<(&'a MappingTable, Vec<&'a Chunk>)>> for Select<'a> {
    fn from(value: Vec<(&'a MappingTable, Vec<&'a Chunk>)>) -> Self {
        Self { inner: value }
    }
}

#[derive(Debug, Clone)]
pub struct Iter<'a, F: WorldFetch> {
    select: Select<'a>,
    pub(crate) iter: Option<(&'a MappingTable, ChunkIter<'a>)>,
    _f: PhantomData<&'a F>,
}

impl<'a, F: WorldFetch> Iter<'a, F> {
    pub fn new<Q: WorldFilter>(world: &mut World) -> Iter<'_, F> {
        let select = Select::new::<F, Q>(world);

        Iter {
            select,
            iter: None,
            _f: PhantomData,
        }
    }
}

impl<'a, F: WorldFetch> Iterator for Iter<'a, F> {
    type Item = F::Item<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.iter.is_none() {
            let (mapping, chunk) = unsafe { std::mem::transmute(self.select.pop()?) };
            let iter = ChunkIter::new(chunk);
            self.iter = Some((mapping, iter));
        }
        let (mapping_table, iter) = self.iter.as_mut()?;
        let Some(components) = iter.next()else{
            
            self.iter  = None;
            return self.next();
        };

        let item = unsafe { F::build(components, mapping_table) };
        Some(item)
    }
}
