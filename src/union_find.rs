use std::marker::PhantomData;

pub trait Key {
    fn make(index: u32) -> Self;
    fn index(&self) -> u32;
}

impl Key for u32 {
    fn make(index: u32) -> Self { index }
    fn index(&self) -> u32 { *self }
}

pub trait Value : Sized {
    fn unify(_val1: &Self, _val2: &Self) -> Option<Self> {
        None
    }
}

#[derive(Debug)]
struct Node<V> {
    parent: u32,
    rank: u32,
    value: V,
}

#[derive(Debug)]
pub struct DisjointSet<K, V> {
    nodes: Vec<Node<V>>,
    phantom: PhantomData<K>
}

impl <K: Key, V: Value> DisjointSet<K, V> {
    pub fn with_capacity(capacity: usize) -> Self {
        Self{
            nodes: Vec::with_capacity(capacity),
            phantom: PhantomData
        }
    }

    pub fn reset(&mut self) {
        self.nodes.clear()
    }

    pub fn add(&mut self, value: V) -> K {
        let parent = self.nodes.len() as u32;
        let rank   = 0;
        let node   = Node { parent, rank, value };
        self.nodes.push( node );
        Key::make( parent )
    }

    fn find_repr_node(&self, key: K) -> usize {
        let mut node = key.index() as usize;
        loop {
            let parent = self.nodes[node].parent as usize;
            match node == parent {
                true  => return node,
                false => node = parent,
            }
        }
    }

    pub fn merge(&mut self, key1: K, key2: K) {
        let rep1 = self.find_repr_node(key1);
        let rep2 = self.find_repr_node(key2);

        if rep1 != rep2 {
            let (lo, hi, rank) = {
                let n1  = unsafe { self.nodes.get_unchecked(rep1) };
                let n2  = unsafe { self.nodes.get_unchecked(rep2) };
                let inc = if n1.rank == n2.rank {1} else {0};
                match n1.rank <= n2.rank {
                    true  => (rep2, rep1, n1.rank + inc),
                    false => (rep1, rep2, n2.rank + inc),
                }
            };

            unsafe {
                let lo : *mut _ = self.nodes.get_unchecked_mut(lo);
                (*lo).parent = hi as u32;
                let hi : *mut _ = self.nodes.get_unchecked_mut(hi);
                (*hi).rank   = rank as u32;
                if let Some(value) = V::unify(&(*lo).value, &(*hi).value) {
                    (*hi).value = value;
                }
            }
        }
    }

    pub fn find(&mut self, key: K) -> &mut V {
        let repr = self.find_repr_node(key);
        unsafe {
            let curr = self.nodes.get_unchecked_mut(repr);
            &mut curr.value
        }
    }
}

#[cfg(test)]
mod tests {
    use super::DisjointSet;
    use std::cmp::min;
    
    #[test]
    fn insert1() {
        impl super::Value for char {
            fn unify(val1: &Self, val2: &Self) -> Option<Self> {
                Some(min(*val1, *val2))
            }
        }
        
        let mut set = DisjointSet::<u32,char>::with_capacity(10);
        let node1 = set.add('1');
        let node2 = set.add('2');
        let node3 = set.add('3');
        let node4 = set.add('4');
        let node5 = set.add('5');
        let node6 = set.add('6');

        assert_eq!(*set.find(node1), '1');
        assert_eq!(*set.find(node2), '2');
        assert_eq!(*set.find(node3), '3');
        assert_eq!(*set.find(node4), '4');
        assert_eq!(*set.find(node5), '5');
        assert_eq!(*set.find(node6), '6');
        
        set.merge(node2, node4);
        assert_eq!(*set.find(node2), '2');
        assert_eq!(*set.find(node4), '2');
        assert_eq!(*set.find(node6), '6');

        set.merge(node4, node6);
        assert_eq!(*set.find(node1), '1');
        assert_eq!(*set.find(node2), '2');
        assert_eq!(*set.find(node3), '3');
        assert_eq!(*set.find(node4), '2');
        assert_eq!(*set.find(node5), '5');
        assert_eq!(*set.find(node6), '2');
    }
}
