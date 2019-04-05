use std::borrow::Borrow;
use std::cmp::Eq;
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::hash::Hash;
use std::mem;

#[derive(Debug)]
struct Inner<K: Hash + Eq, V> {
    scope: u32,
    curr_map: HashMap<K, V>,
    prev_scope: Option<Box<Inner<K, V>>>,
}

#[derive(Debug)]
pub struct ScopedMap<K: Eq + Hash, V> {
    inner: Box<Inner<K, V>>,
}

impl<K: Hash + Eq, V> Inner<K, V> {
    fn get<Q: ?Sized>(&self, k: &Q) -> Option<&V>
    where
        K: Borrow<Q>,
        Q: Hash + Eq,
    {
        match self.curr_map.get(k) {
            None => (),
            v => return v,
        }
        match self.prev_scope {
            None => None,
            Some(ref prev) => prev.get(k),
        }
    }
    fn get_mut<Q: ?Sized>(&mut self, k: &Q) -> Option<&mut V>
    where
        K: Borrow<Q>,
        Q: Hash + Eq,
    {
        match self.curr_map.get_mut(k) {
            None => (),
            v => return v,
        }
        match self.prev_scope {
            None => None,
            Some(ref mut prev) => prev.get_mut(k),
        }
    }
}

impl<K: Hash + Eq, V> ScopedMap<K, V> {
    pub fn new() -> Self {
        let inner = Inner {
            curr_map: HashMap::new(),
            prev_scope: None,
            scope: 0,
        };
        ScopedMap {
            inner: Box::new(inner),
        }
    }

    pub fn scope(&self) -> u32 {
        self.inner.scope
    }
    pub fn begin_scope(&mut self) {
        let inner = Inner {
            curr_map: HashMap::new(),
            prev_scope: None,
            scope: self.inner.scope + 1,
        };
        let old = mem::replace(&mut self.inner, Box::new(inner));
        self.inner.prev_scope = Some(old);
    }

    pub fn end_scope(&mut self) {
        self.inner = self.inner.prev_scope.take().unwrap();
    }

    pub fn insert(&mut self, k: K, v: V) -> Option<V> {
        self.inner.curr_map.insert(k, v)
    }

    pub fn entry(&mut self, k: K) -> Entry<K, V> {
        self.inner.curr_map.entry(k)
    }

    pub fn get<Q: ?Sized>(&self, k: &Q) -> Option<&V>
    where
        K: Borrow<Q>,
        Q: Hash + Eq,
    {
        self.inner.get(k)
    }
    pub fn get_mut<Q: ?Sized>(&mut self, k: &Q) -> Option<&mut V>
    where
        K: Borrow<Q>,
        Q: Hash + Eq,
    {
        self.inner.get_mut(k)
    }
}

#[cfg(test)]
mod tests {
    use super::ScopedMap;

    #[test]
    fn insert1() {
        let mut map = ScopedMap::new();
        map.insert(1, "a");
        assert_eq!(map.get(&1), Some(&"a"));
        assert_eq!(map.get(&2), None)
    }

    #[test]
    fn insert2() {
        let mut map = ScopedMap::new();
        map.insert(1, "a");
        assert_eq!(map.get(&1), Some(&"a"));
        assert_eq!(map.get(&2), None);

        map.begin_scope();
        map.insert(1, "b");
        map.insert(2, "c");
        assert_eq!(map.get(&1), Some(&"b"));
        assert_eq!(map.get(&2), Some(&"c"));

        map.end_scope();
        assert_eq!(map.get(&1), Some(&"a"));
        assert_eq!(map.get(&2), None);
    }
}
