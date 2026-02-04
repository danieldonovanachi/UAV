pub enum Error {
    /// The graph is empty or not **connected**, preventing the formation of an MST
    /// that spans all vertices.
    //#[error("Graph precondition failed: graph must be non-empty and connected.")]
    PreconditionFailed,
}

pub struct MinimumSpanningTree<G>
where
    G: gryf::core::GraphBase,
{
    edges: std::collections::HashSet<G::EdgeId>,
}
