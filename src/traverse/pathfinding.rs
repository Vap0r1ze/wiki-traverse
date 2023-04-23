use indexmap::{map::Entry::Vacant, IndexMap};
use rustc_hash::FxHasher;
use std::hash::{BuildHasherDefault, Hash};

type FxIndexMap<K, V> = IndexMap<K, V, BuildHasherDefault<FxHasher>>;

fn reverse_path<N, V, F>(parents: &FxIndexMap<N, V>, mut parent: F, start: usize) -> Vec<N>
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

pub fn bfs<N, FN, IN, FS>(start: &N, mut successors: FN, mut success: FS) -> Option<Vec<N>>
where
    N: Eq + Hash + Clone,
    FN: FnMut(&N) -> IN,
    IN: IntoIterator<Item = N>,
    FS: FnMut(&N) -> bool,
{
    if success(start) {
        return Some(vec![start.clone()]);
    }
    let mut i = 0;
    let mut parents: FxIndexMap<N, usize> = FxIndexMap::default();
    parents.insert(start.clone(), usize::max_value());
    while let Some((node, _)) = parents.get_index(i) {
        for successor in successors(node) {
            if success(&successor) {
                let mut path = reverse_path(&parents, |&p| p, i);
                path.push(successor);
                return Some(path);
            }
            if let Vacant(e) = parents.entry(successor) {
                e.insert(i);
            }
        }
        i += 1;
    }
    None
}
