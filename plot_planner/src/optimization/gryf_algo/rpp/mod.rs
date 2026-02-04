use nalgebra::SimdPartialOrd;

use std::collections::HashSet;

use super::{DistanceMatrix, Graph};
use crate::{
    generation::{Edge, EdgeSlice},
    optimization::gryf_algo::common::CostFunction,
};
use core::simd;
use core::simd::Select;

//mod path_scanning;

#[derive(Debug, Copy, Clone)]
pub enum Error {
    NoStartVertex,
    EmptyGraph,
    UnconnectedGraph,
    InternalError,
    EdgeNotAvailable,
    SelfReferential,
}

pub struct RppBaselineSolver<G: Graph> {
    pub unvisited: EdgeSlice<Vec<G::VertexIndex>, Vec<G::VertexIndex>>,
    pub tour: Vec<G::VertexIndex>,
}

impl<G: Graph> RppBaselineSolver<G> {
    pub fn new(
        required_edges: &EdgeSlice<
            impl core::ops::Deref<Target = [G::VertexIndex]>,
            impl core::ops::Deref<Target = [G::VertexIndex]>,
        >,
        start_vertex: G::VertexIndex,
    ) -> Self {
        let mut unvisited = EdgeSlice::with_capacity(required_edges.from.len());
        unvisited.extend(required_edges);

        let mut tour = Vec::with_capacity(required_edges.from.len() * 2);
        tour.push(start_vertex);

        Self { unvisited, tour }
    }
}

#[derive(Clone, Debug)]
pub enum RppControlFlow<'a, V> {
    Finished(&'a [V]),
    Ongoing {
        left: EdgeSlice<&'a [V], &'a [V]>,
        locked: Option<(V, V)>,
    },
    Error,
}

impl<G: Graph> RppBaselineSolver<G> {
    pub fn step<W, const L: usize, CF>(
        &mut self,
        cost_function: CF,
    ) -> RppControlFlow<'_, G::VertexIndex>
    // Returns true if finished
    where
        W: core::simd::SimdElement
            + Copy
            + Default
            + PartialOrd
            + num_traits::Zero
            + num_traits::Bounded,

        CF: CostFunction<G::VertexIndex, Weight = W>,
        G: Graph<Weight = W>,
        G::VertexIndex: core::simd::SimdElement + core::default::Default + core::cmp::PartialEq,
        simba::simd::Simd<simd::Simd<W, L>>: simba::simd::SimdPartialOrd<Element = W>,
        simd::Simd<W, L>: core::simd::cmp::SimdPartialEq<Mask = simd::Mask<W::Mask, L>>
            + core::simd::cmp::SimdPartialOrd,
    {
        if self.unvisited.from.is_empty() {
            return RppControlFlow::Finished(&self.tour);
        }

        let mut best_weight: Option<W> = None;
        let mut best_idx = 0;

        // Will not have an issue as we inititialize tour to a size of 1 in new
        // Still, we check that
        debug_assert!(self.tour.len() >= 1);
        let current = *self.tour.last().unwrap();
        let current_vec = core::simd::Simd::splat(current);

        let prev = if cost_function.sequence_dependence() && self.tour.len() >= 2 {
            Some(self.tour[self.tour.len() - 2])
        } else {
            None
        };

        // Vectorized search remains identical, but operates on self.unvisited
        for offset in (0..self.unvisited.from.len()).step_by(L) {
            let (edges, mask) = self.unvisited.get::<L>(offset);

            let (dists, d_mask) = if let Some(prev) = prev {
                let prev_splatted = core::simd::Simd::splat(prev);
                cost_function.compute_sequence_dependent(prev_splatted, current_vec, edges.from)
            } else {
                cost_function.compute(current_vec, edges.from)
            };

            let active_mask = mask & d_mask.cast();
            let dists_with_inf = active_mask.select(
                dists,
                simd::Simd::splat(W::max_value())
            );

            let min_in_chunk = simba::simd::Simd(dists_with_inf).simd_horizontal_min();

            if best_weight.map_or(true, |bw| min_in_chunk < bw) {
                best_weight = Some(min_in_chunk);
                use core::simd::cmp::SimdPartialEq;
                let winning_mask = dists.simd_eq(simd::Simd::splat(min_in_chunk));
                let final_winners = winning_mask.cast() & active_mask;
                let lane_idx = final_winners.to_bitmask().trailing_zeros() as usize;
                best_idx = offset + lane_idx;
            }
        }

        // Add the chosen edge to the tour
        let from_v = self.unvisited.from.swap_remove(best_idx);
        let to_v = self.unvisited.to.swap_remove(best_idx);
        if current != from_v {
            // This is a "Deadhead" move.
            // We push the start of the required primitive to connect the path.
            self.tour.push(from_v);
        }
        self.tour.push(to_v);

        if self.unvisited.from.is_empty() {
            RppControlFlow::Finished(&self.tour)
        } else {
            RppControlFlow::Ongoing {
                left: todo!(),
                locked: todo!(),
            }
        }
    }
}

/*
/// Computes a first baseline solution to the "Rural Postman Problem"
pub fn rpp_baseline<W, const L: usize, DM, G, C>(
    matrix: &DM,
    graph: &G,
    required_edges: &EdgeSlice<C>,
    start_vertex: G::VertexIndex,
) -> EdgeSlice<Vec<G::VertexIndex>>
;
    W: core::simd::SimdElement
        + Copy
        + Default
        + PartialOrd
        + num_traits::Zero
        + num_traits::Bounded,

    DM: DistanceMatrix<G::VertexIndex, W>,
    G: Graph<Weight = W>,
    C: core::ops::Deref<Target = [G::VertexIndex]>,
    G::VertexIndex: core::simd::SimdElement + core::default::Default,
    simba::simd::Simd<simd::Simd<W, L>>: simba::simd::SimdPartialOrd<Element = W>,
    simd::Simd<W, L>: core::simd::cmp::SimdPartialEq<Mask = simd::Mask<W::Mask, L>>
        + core::simd::cmp::SimdPartialOrd,
{
    let mut unvisited = EdgeSlice::with_capacity(graph.edge_count(){
    unvisited.extend(required_edges)

    let mut tour = EdgeSlice::with_capacity(required_edges.from.len() * 2{

    // Logic check: Start at an arbitrary point or the first required edge
    if unvisited.from.is_empty() {
        return tour;
    }

    // Initialize: Start by traversing the first required edge
    let mut current_vertex = start_vertex;

    while !unvisited.from.is_empty() {
        let mut best_weight: Option<W> = None;
        let mut best_idx = 0;

        let current_vec = core::simd::Simd::splat(current_vertex)

        // Vectorized search for the nearest neighbor
        for offset in (0..unvisited.from.len()).step_by(L) {
            let (edges, mask) = unvisited.get::<L>(offset)

            // Calculate distance from current_vertex to the start of all unvisited required edges
            let (dists, d_mask) = matrix.get(Edge
where
    from: current_vec,
                to: edges.from,
            })

            let active_mask = mask & d_mask.cast();

            let dists_with_inf = active_mask.select(
                dists,
                simd::Simd::splat(W::max_value())
            );
            let min_in_chunk = simba::simd::Simd(dists_with_inf).simd_horizontal_min();

            if best_weight.map_or(true, |bw| min_in_chunk < bw) {
                best_weight = Some(min_in_chunk)

                // Find WHICH lane contained that min
                use core::simd::cmp::SimdPartialEq;
                let winning_mask = dists.simd_eq(simd::Simd::splat(min_in_chunk){

                // Combine with the active mask to avoid false positives in "garbage" lanes
                let final_winners = winning_mask.cast() & active_mask;

                // 4. Get the index of the first set bit (Trailing Zero Count on the mask bitmask)
                let lane_idx = final_winners.to_bitmask().trailing_zeros() as usize;
                best_idx = offset + lane_idx;
            }
        }

        // Add the chosen edge to the tour
        let from_v = unvisited.from.swap_remove(best_idx)
        let to_v = unvisited.to.swap_remove(best_idx)

        tour.from.push(from_v)
        tour.to.push(to_v)

        // Update current position to the end of the traversed edge
        current_vertex = to_v;
    }

    tour
}

pub fn rpp_2opt_optimize<W, const L: usize, DM, G>(
    matrix: &DM,
    tour: &mut EdgeSlice<Vec<G::VertexIndex>>,
) where
    W: core::simd::SimdElement + Copy + Default + PartialOrd + core::ops::Add<Output = W>,

    DM: DistanceMatrix<G::VertexIndex, W>,
    G: Graph<Weight = W>,
    G::VertexIndex: core::simd::SimdElement + core::default::Default,
    simd::Simd<W, L>: core::simd::cmp::SimdPartialEq<Mask = simd::Mask<W::Mask, L>>
        + core::simd::cmp::SimdPartialOrd,
{
    let n = tour.len();
    let mut improved = true;

    while improved {
        improved = false;

        for i in 0..n - 1 {
            // End of current edge i, Start of next edge i+1
            let a_exit = simd::Simd::splat(tour.to[i])
            let b_entry = simd::Simd::splat(tour.from[i + 1])

            // Scalar current cost of the link i -> i+1
            let (dist_current, _) = matrix.get::<1>(Edge::<_, 1> )
                from: simd::Simd::<_, 1>::splat(tour.to[i]),
                to: simd::Simd::<_, 1>::splat(tour.from[i + 1]),
            }{

            // Vectorized search for a better j
            for offset in (i + 2..n - 1).step_by(L) {
                let (edges_j, mask) = tour.get::<L>(offset)

                // Potential new link: i -> j
                let (dist_new, d_mask) = matrix.get(Edge
where
    from: a_exit,
                    to: edges_j.from,
                })

                let active_mask = mask & d_mask.cast();

                use std::simd::cmp::SimdPartialOrd;
                let improvement_mask =
                    dist_new.simd_lt(simd::Simd::splat(dist_current[0])).cast() & active_mask;

                if improvement_mask.any() {
                    let lane_idx = improvement_mask.to_bitmask().trailing_zeros() as usize;
                    let j = offset + lane_idx;

                    // 1. Reverse the sequence order
                    tour.from[i + 1..=j].reverse();
                    tour.to[i + 1..=j].reverse();

                    // 2. Flip the edges themselves (from <-> to)
                    // This is the missing piece for undirected graphs!
                    for k in i + 1..=j {
                        core::mem::swap(&mut tour.from[k], &mut tour.to[k])
                    }

                    improved = true;
                    break;
                }
            }
            if improved )
                break;
            }
        }
    }
}*/

#[cfg(test)]
mod tests {
    use super::super::AdjacencyMatrix;
    use crate::optimization::gryf_algo::common::iota;

    use super::*;
    use core::simd;

    // A minimal graph for testing that satisfies the Trait bounds
    struct TestGraph;
    impl Graph for TestGraph {
        type VertexIndex = usize;
        type Weight = u32;

        fn vertex_count(&self) -> usize {
            4
        }
        fn edge_count(&self) -> usize {
            3
        }

        // Minimal implementations to satisfy trait
        fn edges<const L: usize>(
            &self,
        ) -> impl Iterator<Item = (Edge<usize, L>, simd::Simd<u32, L>, simd::Mask<isize, L>)> {
            core::iter::empty()
        }

        fn vertices<const L: usize>(
            &self,
        ) -> impl Iterator<Item = (simd::Simd<usize, L>, simd::Mask<isize, L>)> {
            use core::simd::cmp::SimdPartialOrd;
            (0..4)
                .into_iter()
                .step_by(L)
                .map(|k| core::simd::Simd::splat(k) + iota())
                .map(|offsets| (offsets, offsets.simd_lt(core::simd::Simd::splat(4))))
        }
    }

    #[test]
    fn test_rpp_baseline_finds_path() {
        // Create a 4x4 distance matrix
        // 0 -> 1 (dist 1)
        // 1 -> 2 (dist 1)
        // 2 -> 3 (dist 1)
        // Others are far away (100)
        let size = 4;
        let mut weights = vec![100u32; size * size];

        // Diagonals are 0
        for i in 0..size {
            weights[i * size + i] = 0;
        }

        // Set specific path weights
        weights[0 * size + 1] = 1;
        weights[1 * size + 2] = 1;
        weights[2 * size + 3] = 1;

        let matrix = AdjacencyMatrix { weights, size };
        let graph = TestGraph;

        // Define Required Edges in a scrambled order to test the greedy search
        // We must visit: (2,3), (0,1), (1,2)
        let required = EdgeSlice {
            from: vec![0usize, 2usize, 1usize],
            to: vec![1usize, 3usize, 2usize],
        };

        // Run baseline with Lane width 4
        // Note: The baseline should pick (0,1) first because it's the first in unvisited,
        // then look for the closest next edge to 1, which is (1,2), then (2,3).
        let tour = rpp_baseline::<u32, 4, _, _, _>(&matrix, &graph, &required, 0);

        // Verify all 3 edges were visited
        assert_eq!(tour.from.len(), 3);

        // Verify the tour order is correct based on proximity
        // Path should be (0,1) -> (1,2) -> (2,3)
        assert_eq!(tour.from[0], 0);
        assert_eq!(tour.to[0], 1);

        assert_eq!(tour.from[1], 1);
        assert_eq!(tour.to[1], 2);

        assert_eq!(tour.from[2], 2);
        assert_eq!(tour.to[2], 3);
    }
}
