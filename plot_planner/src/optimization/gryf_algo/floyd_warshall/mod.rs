use super::{DistanceMatrix, Graph};
use crate::generation::Edge;
use ndarray::Array2;
use core::simd;
use core::simd::Select;

/// Used for APSP (All Pairs Shortest Path)
pub struct RoutingTable {
    pub distances: Array2<f64>,
    // next_hop[i, j] stores the next vertex to visit to get from i to j
    pub next_hop: Array2<Option<usize>>,
}

// https://moorejs.github.io/APSP-in-parallel/
pub fn floyd_warshall_simd<W, const L: usize, DM, G>(matrix: &mut DM, graph: &G)
where
    W: core::simd::SimdElement
        + Copy
        + Default
        + PartialOrd
        + core::ops::Add<Output = W>
        + num_traits::Zero,

    DM: DistanceMatrix<G::VertexIndex, W>,
    // The weight needs to be summable & comparable
    core::simd::Simd<W, L>: core::ops::Add<Output = core::simd::Simd<W, L>>
        + core::simd::cmp::SimdPartialOrd<Mask = core::simd::Mask<W::Mask, L>>,
    core::simd::Simd<usize, L>: core::ops::Add<Output = core::simd::Simd<usize, L>>,
    G: Graph<Weight = W>,
{
    // First we insert the weight of the edges that are directly connected
    for (edge, weight, mask) in graph.edges::<1>() {
        matrix.set(edge, weight, mask);
    }

    // Then, for all the vertices, we set the edges from themselves to themselves to 0
    for (vertices, mask) in graph.vertices::<1>() {
        let edge = Edge {
            from: vertices,
            to: vertices,
        };
        let weights = core::simd::Simd::splat(W::zero());
        matrix.set(edge, weights, mask);
    }

    // The k & i iterations are scalar
    for (k, _) in graph.vertices::<1>() {
        let k_vec = core::simd::Simd::splat(k[0]);;

        for (i, _) in graph.vertices::<1>() {
            let i_vec = core::simd::Simd::splat(i[0]);
            let (ik_weight, _) = matrix.get::<1>(Edge::<_, 1> { from: i, to: k });
            let ik_weights = core::simd::Simd::splat(ik_weight[0]);

            // The j iteration is vectorized
            for (j_vec, j_mask) in graph.vertices::<L>() {
                // Get current d[i][j]
                let (ij_weights, ij_mask) = matrix.get(Edge {
                    from: i_vec,
                    to: j_vec,
                });

                // Get current d[k][j]
                let (kj_weights, kj_mask) = matrix.get(Edge {
                    from: k_vec,
                    to: j_vec,
                });

                // Relax: d[i][j] = min(d[i][j], d[i][k] + d[k][j])
                let sum = ik_weights + kj_weights;

                // Use the mask to ensure we only update valid vertices
                let combined_mask = ij_mask & kj_mask & j_mask;
                let lt_mask = core::simd::cmp::SimdPartialOrd::simd_lt(sum, ij_weights);
                // Convert mask to isize for select method
                let lt_mask_isize: simd::Mask<isize, L> = lt_mask.cast();
                let new_min = lt_mask_isize.select(sum, ij_weights);

                // Update the matrix
                matrix.set(
                    Edge {
                        from: i_vec,
                        to: j_vec,
                    },
                    new_min,
                    combined_mask,
                );
            }
        }
    }
}
