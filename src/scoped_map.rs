use std::collections::HashMap;
use std::mem;
use std::borrow::Borrow;
use std::hash::Hash;
use std::cmp::Eq;

struct Inner<K,V> {
    curr_map: HashMap<K,V>,
    prev_scope: Option<Box<Inner<K,V>>>,
}

pub struct ScopedMap<K,V> {
    inner: Box<Inner<K,V>>,
}


impl <K: Hash + Eq,V> ScopedMap<K,V> {
    pub fn new() -> Self {
        let inner = Inner{curr_map: HashMap::new(), prev_scope: None };
        ScopedMap{ inner: Box::new(inner) }
    }

    pub fn begin_scope(&mut self) {
        let inner = Inner{curr_map: HashMap::new(), prev_scope: None };
        let old = mem::replace(&mut self.inner, Box::new(inner));
        self.inner.prev_scope = Some(old);
    }

    pub fn end_scope(&mut self) {
        self.inner = self.inner.prev_scope.take().unwrap();
    }

    pub fn insert(&mut self, k: K, v: V) -> Option<V> {
        self.inner.curr_map.insert(k, v)
    }

    pub fn get<Q: ?Sized>(&self, k: &Q) -> Option<&V>  where
        K: Borrow<Q>,
        Q: Hash + Eq
    {
        match self.inner.curr_map.get(k) {
            None => {
                match self.inner.prev_scope {
                    None => None,
                    Some(ref prev) => (**prev).curr_map.get(k)
                }
            },
            v => v
        }
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
