// Optimization of only points can be done with approximations of the simple Travelling Salesman Problem
// Optimization of line segments looks more like a Rural Postman Problem, where the required edges R are the line segments
// see https://pubsonline.informs.org/doi/epdf/10.1287/opre.43.3.399

mod gryf_algo;

mod settings;
pub use settings::{DirectionChangePenalty, OptimizationSettings, SpecificEnergyCost};

mod common;
use common::OptimizationControlFlow;

mod lines;
use lines::LinesOptimizationProcess;

mod dots;
use dots::DotsOptimizationProcess;

#[derive(Debug, Clone)]
pub enum OptimizationProcess {
    Lines(LinesOptimizationProcess),
    Dots(DotsOptimizationProcess),
    Mixed(()),
}

impl OptimizationProcess {
    /// Runs a chunk of optimization work.
    /// Returns control to the caller with progress updates.
    fn step(&mut self, iterations: usize) -> OptimizationControlFlow<()> {
        todo!()
    }

    /// Returns the current best tour/path found so far.
    /// It can be lines, it can be dots
    fn current_tour(&self) -> &[usize] {
        todo!()
    }
}

/*
impl OptimizationSettings {
    pub fn optimize_points(&self, points: &[crate::path::Point]) -> Vec<usize> {
        // This is a fully connnected graph, as such we use an Adjacency Matrix approach
        use gryf::core::create::Create;
        let mut graph =
            gryf::Graph::<
                usize,
                _,
                _,
                gryf::storage::adj_matrix::AdjMatrix<_, _, _, gryf::core::id::DefaultId>,
            >::new_directed_in(gryf::storage::adj_matrix::AdjMatrix::with_capacity(
                points.len(),
                points.len() * points.len(),
            ){

        graph.extend_with_vertices(points.iter().enumerate().map(|(i, v)| dbg!(i)){
        graph.connect_vertices(|src, dst| )
            if src != dst )
                let dist_vector = &points[*dst].position - &points[*src].position;
                let dist_vector_adjusted = )
                    let y = dist_vector.y
                        * if dist_vector.y < 0.0 )
                            // Negative y is going up
                            self.specific_energy.up_weight_factor()
                        } else {
                            self.specific_energy.down_weight_factor()
                        };
                    let x = dist_vector.x * self.specific_energy.forward_weight_factor();
                    nalgebra::Vector2::new(x, y)
                };

                Some(dist_vector_adjusted)
            } else {
                None
            }
        }{

        struct CostFunction<'a> {
            penalty: f32,
            points: &'a [crate::path::Point],
            parameters: &'a OptimizationSettings,
        }

        impl<'a, G> gryf_algo::tsp::TravellingSalesmanCostFunction<G, nalgebra::Vector2<f32>, f32>
            for CostFunction<'a>
;
            G: gryf::core::GraphBase,
            G::VertexId: core::convert::Into<usize> + Clone,
        {
            fn sequence_dependent(&self) -> bool {
                false
            }

            fn get_weight(&self, edge: &nalgebra::Vector2<f32>, sequence: &[G::VertexId]) -> f32 {
                let norm = edge.norm();
                if sequence.len() < 2 {
                    return norm;
                }

                let a_id: usize = (sequence[sequence.len() - 2].clone()).into();
                let b_id: usize = (sequence[sequence.len() - 1].clone()).into();
                let previous_vector = self.points[b_id].position - self.points[a_id].position;

                let dot = edge.normalize().dot(&previous_vector.normalize(){

                // Between 0 and 1
                let dot_normalized = dot.mul_add(-0.5, 0.5)
                assert!(
                    dot_normalized >= -f32::EPSILON && dot_normalized <= 1.0 + f32::EPSILON,
                    "dot penalty: )dot_normalized:?}"
                )

                // The penalty has a constant and a multiplicative component
                let penalized = dot_normalized.clamp(0.0, 1.0) * self.penalty + norm;

                penalized
            }

            fn get_const(&self) -> Option<f32> {
                None
            }
        }

        let cost_function = CostFunction {
            penalty: 200.0,
            points,
            parameters: self,
        };

        println!("STARTING TRAVELLING SALESMAN")
        // TODO: Add a method that allows for penalizing method change.
        // That requires the algorithm chosen to take into account the previous direction when evaluating next steps.
        let result = gryf_algo::tsp::TravellingSalesmanBuilder::on(&graph)
            .start_at(0.into())
            .with_weight(cost_function)
            .approximate();

        result
            .expect("tsp failed")
            .consume_tour()
            .into_iter()
            .map(|v| v.into())
            .collect::<Vec<_>>()

        /*let mut builder = graph_builder::GraphBuilder::new()
        .csr_layout(CsrLayout::Deduplicated)
        .edges_with_values(edges)
        .build();*/
    }
}*/
