use std::marker::PhantomData;
use std::cmp::min;
use std::usize;

struct Vertex<Data> {
    data: Data,
    edges: Vec<u32>,
}

pub trait VertexKey {
    fn make(index: u32) -> Self;
    fn index(&self) -> u32;
}

impl VertexKey for u32 {
    fn make(index: u32) -> Self { index }
    fn index(&self) -> u32 { *self }
}

pub struct Graph<K, Data> {
    phantom: PhantomData<K>,
    vertices: Vec<Vertex<Data>>,
}

impl <K, Data> Graph<K, Data> where K: VertexKey {
    pub fn new() -> Self {
        Graph {
            vertices: vec![],
            phantom: PhantomData
        }
    }

    pub fn add_vertex(&mut self, data: Data) -> K {
        let index  = self.vertices.len() as u32;
        let edges  = Vec::new();
        let vertex = Vertex{ data, edges };
        self.vertices.push(vertex);
        K::make(index)
    }

    pub fn add_edge(&mut self, v1: K, v2: K) {
        let v1 = v1.index() as usize;
        let v2 = v2.index();
        self.vertices[v1].edges.push(v2);
    }
}

pub struct SCC<'a, K: 'a, Data: 'a> {
    graph: &'a Graph<K, Data>,
    stack: Vec<usize>,
    indices: Vec<usize>,
    lowlink: Vec<usize>,
    onstack: Vec<bool>,
    result:  Vec<Vec<Data>>,
    curr_index: usize,
}

impl <'a, K: VertexKey, Data: Clone> SCC<'a, K, Data> {
    pub fn run(graph: &Graph<K,Data>) -> Vec<Vec<Data>>{
        let len = graph.vertices.len();
        let mut indices = Vec::with_capacity(len);
        let mut lowlink = Vec::with_capacity(len);
        let mut onstack = Vec::with_capacity(len);
        let stack   = Vec::with_capacity(len);
        let result  = Vec::with_capacity(len);

        for _ in 0..len {
            indices.push(usize::MAX);
            lowlink.push(usize::MAX);
            onstack.push(true);
        };
        let curr_index = 0;
        let mut scc = SCC {
            graph,
            stack,
            indices,
            lowlink,
            onstack,
            result,
            curr_index
        };
        for v in 0..len {
            if !scc.visited(v) {
                scc.scc(v);
            }
        }

        scc.result
    }

    fn visited(&self, v: usize) -> bool {
        self.indices[v] != usize::MAX
    }

    fn scc(&mut self, v: usize) {
        let index = self.curr_index;
        self.indices[v] = index;
        self.lowlink[v] = index;
        self.onstack[v] = true;
        self.stack.push(v);
        self.curr_index += 1;

        for w in &self.graph.vertices[v as usize].edges {
            let w = *w as usize;
            if !self.visited(w as usize) {
                self.scc(w);
                self.lowlink[v] = min(self.lowlink[v], self.lowlink[w]);
            }
            else if self.onstack[w] {
                // Successor w is in stack S and hence in the current SCC
                // If w is not on stack, then (v, w) is a cross-edge in the
                //    DFS tree and must be ignored
                self.lowlink[v] = min(self.lowlink[v], self.indices[w]);
            }
        }

        //if v is a root node, pop the stack and generate an SCC
        if self.indices[v] == self.lowlink[v] {
            let mut scc = Vec::new();
            while let Some(w) = self.stack.pop() {
                scc.push(self.graph.vertices[w].data.clone());
                if w == v { break }
            }
            self.result.push(scc)
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sort() {
        let mut graph = Graph::<u32,u32>::new();

        let v1 = graph.add_vertex(1);
        let v2 = graph.add_vertex(2);
        let v3 = graph.add_vertex(3);
        let v4 = graph.add_vertex(4);
        let v5 = graph.add_vertex(5);
        let v6 = graph.add_vertex(6);
        let v7 = graph.add_vertex(7);
        let v8 = graph.add_vertex(8);
        let v9 = graph.add_vertex(9);
        let v10 = graph.add_vertex(10);

        graph.add_edge(v1, v2);
        graph.add_edge(v2, v3);
        graph.add_edge(v3, v4);
        graph.add_edge(v3, v5);
        graph.add_edge(v5, v6);
        graph.add_edge(v6, v7);
        graph.add_edge(v7, v8);
        graph.add_edge(v8, v9);
        graph.add_edge(v9, v7);
        graph.add_edge(v7, v3);
        graph.add_edge(v7, v10);

        let res = SCC::run(&graph);

        println!("{:#?}", res);

    }
}
