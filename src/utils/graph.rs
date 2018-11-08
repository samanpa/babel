

trait Graph {
    type Vertex;
    type Vertices: Iterator<Item=Self::Vertex>;

    fn adjacent(&self, v: &Vertex) -> Self::Vertices;
}
