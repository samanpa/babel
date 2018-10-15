use std::num::NonZeroU32;

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub struct NodeId(pub (super) usize);

#[derive(Debug)]
pub struct Node<T> {
    parent: NodeId,
    rank: NonZeroU32,
    value: T,
}

#[derive(Debug)]
pub struct DisjointSet<T> {
    nodes: Vec<Node<T>>,
}

impl <T> DisjointSet<T> {
    pub fn new() -> Self {
        Self{ nodes: Vec::new() }
    }

    pub fn next_node_id(&self) -> NodeId {
        NodeId(self.nodes.len())
    }

    pub fn add(&mut self, value: T) -> NodeId {
        let parent = self.next_node_id();
        let rank   = unsafe { NonZeroU32::new_unchecked(1) };
        let node   = Node { parent, rank, value };
        self.nodes.push( node );
        parent
    }

    fn find_tc(&self, node: NodeId) -> NodeId {
        let parent = unsafe{ self.nodes.get_unchecked(node.0).parent };
        match node == parent {
            false => self.find_tc(parent),
            true  => node
        }
    }

    pub fn get_repr(&self, node_id: NodeId) -> NodeId {
        self.find_tc(node_id)
    }

    fn get(&mut self, node_id: NodeId) -> &mut T {
        unsafe {
            let curr = self.nodes.get_unchecked_mut(node_id.0 as usize);
            &mut curr.value
        }
    }

    pub fn merge(&mut self, node1: NodeId, node2: NodeId) -> &mut T {
        let rep1 = self.get_repr(node1);
        let rep2 = self.get_repr(node2);

        let (nodeid, rank) = if rep1 == rep2 {
            (rep2, None)
        }
        else {
            let (lo, hi, rank) = {
                let n1  = unsafe { self.nodes.get_unchecked(rep1.0) };
                let n2  = unsafe { self.nodes.get_unchecked(rep2.0) };
                let inc = if n1.rank.get() == n2.rank.get() {1} else {0};
                match n1.rank <= n2.rank {
                    true  => (rep2, rep1, n1.rank.get() + inc),
                    false => (rep1, rep2, n2.rank.get() + inc),
                }
            };

            let lo = unsafe { self.nodes.get_unchecked_mut(lo.0) };
            lo.parent = hi;
            (hi, Some(rank))
        };

        let node = unsafe { self.nodes.get_unchecked_mut(nodeid.0) };
        if let Some(rank) = rank {
            node.rank =  unsafe { NonZeroU32::new_unchecked(rank) };
        }
        &mut node.value
    }

    pub fn find(&mut self, node_id: NodeId) -> &mut T {
        let repr = self.get_repr(node_id);
        self.get(repr)
    }
}

#[cfg(test)]
mod tests {
    use super::DisjointSet;
    
    #[test]
    fn insert1() {
        let mut set = DisjointSet::new();
        let node1 = set.add('1');
        let node2 = set.add('2');
        let node3 = set.add('3');
        let node4 = set.add('4');
        let node5 = set.add('5');

        assert_eq!(*set.find(node1), '1');
        assert_eq!(*set.find(node2), '2');
        assert_eq!(*set.find(node3), '3');
        assert_eq!(*set.find(node4), '4');
        assert_eq!(*set.find(node5), '5');

        set.merge(node2, node4);
        
        assert_eq!(*set.find(node2), '2');
        assert_eq!(*set.find(node4), '2');
    }
}
