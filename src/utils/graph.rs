use std::marker::PhantomData;

pub(super) struct Vertex<Data> {
    pub(super) data: Data,
    pub(super) edges: Vec<u32>,
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
    pub(super) vertices: Vec<Vertex<Data>>,
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

