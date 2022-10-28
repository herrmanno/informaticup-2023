struct PathGrah {
    /// Vec with fields as indices to vector of edges that start at that field
    graph: Vec<Vec<usize>>,
    /// All edges
    edges: Vec<Edge>,
    /// Vector with fields as indices to vector of edge indices, that use the field
    /// Essentially `field -> Edges`
    fields_used_by_edges: Vec<Vec<usize>>,
    /// The edges that define the *taken* paths in this graph
    edges_active: Vec<usize>,
    /// Edges that can not be taken because some fields are already in used, that they rely on
    edges_disabled: HashSet<usize>,
}

struct Edge {
    to: usize,
    fields_used: Vec<usize>,
    // more data
}