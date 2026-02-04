mod nearest_neighbor;

#[derive(Debug, Copy, Clone)]
pub enum Error {
    NoStartVertex,
    EmptyGraph,
    UnconnectedGraph,
    InternalError,
    EdgeNotAvailable,
    SelfReferential,
}

/// The result of a Traveling Salesperson Problem computation.
#[derive(Debug, Clone)]
pub struct TspPath<W, G>
where
    G: gryf::core::GraphBase,
{
    /// The sequence of vertices visited in the calculated tour.
    tour: Vec<G::VertexId>,
    /// The total weight (length) of the tour.
    total_weight: W,
}

impl<W, G> TspPath<W, G>
where
    G: gryf::core::GraphBase,
{
    pub fn consume_tour(self) -> Vec<G::VertexId> {
        self.tour
    }

    /// Returns the sequence of vertices in the tour.
    pub fn tour(&self) -> &Vec<G::VertexId> {
        &self.tour
    }

    /// Returns the total calculated weight of the tour.
    pub fn total_weight(&self) -> &W {
        &self.total_weight
    }
}

pub trait TravellingSalesmanCostFunction<G, E, W>
where
    G: gryf::core::GraphBase,
    W: gryf::core::weight::Weight,
{
    /// If this cost function is sequence dependent
    fn sequence_dependent(&self) -> bool;
    fn get_weight(&self, edge: &E, sequence: &[G::VertexId]) -> W;
    fn get_const(&self) -> Option<W>;
}

pub struct WeightCostFunction<F>(F);

impl<F, G, E, W> TravellingSalesmanCostFunction<G, E, W> for WeightCostFunction<F>
where
    G: gryf::core::GraphBase,
    F: gryf::core::weight::GetWeight<E, W>,
    W: gryf::core::weight::Weight,
{
    fn sequence_dependent(&self) -> bool {
        false
    }

    fn get_weight(&self, edge: &E, _sequence: &[G::VertexId]) -> W {
        self.0.get(edge)
    }

    fn get_const(&self) -> Option<W> {
        self.0.get_const()
    }
}

/// The entry point for computing a TSP tour.
pub struct TravellingSalesmanBuilder<'a, W, G, F>
where
    G: gryf::core::GraphBase,
{
    graph: &'a G,
    start_vertex: Option<G::VertexId>,
    edge_weight: F,
    // Optional: Algorithm choice (only NearestNeighbor for now)
    _phantom_w: std::marker::PhantomData<W>,
}

impl<'a, W, G> TravellingSalesmanBuilder<'a, W, G, WeightCostFunction<gryf::core::weight::Identity>>
where
    G: gryf::core::GraphBase,
{
    /// Constructs a new builder for the given graph.
    pub fn on(graph: &'a G) -> Self {
        TravellingSalesmanBuilder {
            graph,
            start_vertex: None,
            edge_weight: WeightCostFunction(gryf::core::weight::Identity),
            _phantom_w: std::marker::PhantomData,
        }
    }
}

impl<'a, W, G, F> TravellingSalesmanBuilder<'a, W, G, F>
where
    G: gryf::core::GraphBase,
{
    pub fn with_weight<F2>(self, f: F2) -> TravellingSalesmanBuilder<'a, W, G, F2> {
        TravellingSalesmanBuilder {
            graph: self.graph,
            start_vertex: self.start_vertex,
            edge_weight: f,
            _phantom_w: self._phantom_w,
        }
    }

    /// Sets the starting vertex for the tour. Required for Nearest Neighbor.
    pub fn start_at(mut self, start: G::VertexId) -> Self {
        self.start_vertex = Some(start);
        self
    }

    /// Executes an approximate run
    pub fn approximate<V, E>(self) -> Result<TspPath<W, G>, Error>
    where
        G: gryf::core::GraphBase
            + gryf::core::Neighbors
            + gryf::core::VertexSet
            + gryf::core::EdgeSet
            + gryf::core::GraphWeak<V, E>,
        W: gryf::core::weight::Weight + PartialOrd + Clone,
        G::VertexId: Eq + std::hash::Hash + Clone,
        F: TravellingSalesmanCostFunction<G, E, W>,
    {
        let start_v = self.start_vertex.ok_or(Error::NoStartVertex)?;

        // Delegate to the core implementation.
        nearest_neighbor::nearest_neighbor(self.graph, start_v, self.edge_weight)
    }
}
