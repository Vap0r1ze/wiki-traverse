use indexmap::{map::Entry::Vacant, IndexMap};
use redis::Commands;
use rustc_hash::FxHasher;
use serde::{Deserialize, Serialize};
use std::{
    fmt::Display,
    hash::{BuildHasherDefault, Hash},
    sync::Mutex,
};

use crate::cache;

type FxIndexMap<K, V> = IndexMap<K, V, BuildHasherDefault<FxHasher>>;
type Node = String;

fn collect_path<N, V, F>(parents: &FxIndexMap<N, V>, mut parent: F, start: usize) -> Vec<N>
where
    N: Eq + Hash + Clone,
    F: FnMut(&V) -> usize,
{
    let mut i = start;
    let path = std::iter::from_fn(|| {
        parents.get_index(i).map(|(node, value)| {
            i = parent(value);
            node
        })
    })
    .collect::<Vec<&N>>();

    path.into_iter().rev().cloned().collect()
}

pub struct TraversalIterator {
    // source: Node,
    target: Node,
    cache_guard: Mutex<redis::Connection>,
    parents: FxIndexMap<Node, usize>,
    i: usize,
}
impl TraversalIterator {
    pub fn new(source: Node, target: Node, cache: redis::Connection) -> Self {
        // if success(source) {
        //     return Some(vec![source.clone()]);
        // }
        let mut parents = FxIndexMap::default();
        parents.insert(source, usize::max_value());

        Self {
            target,
            cache_guard: Mutex::new(cache),
            parents,
            i: 0,
        }
    }
    pub fn from_names(
        source_name: &str,
        target_name: &str,
        mut cache: redis::Connection,
    ) -> Result<Self, String> {
        let target_id: String = cache
            .get(cache::IdOf(target_name))
            .map_err(|_| format!("Could not find target page {target_name:?}"))?;
        let source_id: String = cache
            .get(cache::IdOf(source_name))
            .map_err(|_| format!("Could not find source page {source_name:?}"))?;
        Ok(Self::new(source_id, target_id, cache))
    }
    fn successors(&self, node: &Node) -> Vec<Node> {
        let mut cache = self.cache_guard.lock().unwrap();
        let link_ids: Vec<String> = cache.smembers(cache::Links(node)).unwrap();
        link_ids
    }
}
impl Iterator for TraversalIterator {
    type Item = Vec<Node>;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some((node, _)) = self.parents.get_index(self.i) {
            for successor in self.successors(node) {
                if successor == self.target {
                    let mut path = collect_path(&self.parents, |&p| p, self.i);
                    path.push(successor);
                    self.i += 1;
                    return Some(path);
                }
                if let Vacant(e) = self.parents.entry(successor) {
                    e.insert(self.i);
                }
            }
            self.i += 1;
        }
        None
    }
}

#[derive(Serialize, Deserialize)]
pub struct Page {
    pub id: String,
    pub name: String,
    pub aliases: Vec<String>,
}

pub struct PathIterator {
    inner: TraversalIterator,
}
impl PathIterator {
    pub fn new(
        source_name: &str,
        target_name: &str,
        cache: redis::Connection,
    ) -> Result<Self, String> {
        let inner = TraversalIterator::from_names(source_name, target_name, cache)?;
        Ok(Self { inner })
    }
}
impl Iterator for PathIterator {
    type Item = Path;
    fn next(&mut self) -> Option<Self::Item> {
        let page_ids = match self.inner.next() {
            Some(path) => path,
            None => return None,
        };

        let mut cache = self.inner.cache_guard.lock().unwrap();

        let names: Vec<String> = cache.mget(&page_ids).unwrap();
        let page_aliases: Vec<Vec<String>> = page_ids
            .iter()
            .map(|id| {
                let alias_ids: Vec<String> = cache.smembers(cache::Aliases(id)).unwrap();
                if alias_ids.is_empty() {
                    vec![]
                } else {
                    cache.mget(&alias_ids).unwrap()
                }
            })
            .collect();
        let pages = names
            .iter()
            .zip(page_ids.iter())
            .zip(page_aliases.iter())
            .map(|((name, id), aliases)| Page {
                id: id.to_string(),
                name: name.to_string(),
                aliases: aliases.clone(),
            })
            .collect::<Vec<_>>();

        Some(pages.into())
    }
}

pub struct Path {
    inner: Vec<Page>,
}
impl Path {
    pub fn take_inner(self) -> Vec<Page> {
        self.inner
    }
}
impl Display for Path {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let names = self
            .inner
            .iter()
            .map(|p| p.name.as_str())
            .collect::<Vec<_>>();
        write!(f, "{}", names.join(" -> "))?;
        Ok(())
    }
}
impl From<Vec<Page>> for Path {
    fn from(pages: Vec<Page>) -> Self {
        Self { inner: pages }
    }
}
