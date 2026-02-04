use super::common::DistanceMatrix;
use crate::generation::Edge;
use core::simd;

// A simple Adjacency Matrix that implements DistanceMatrix
#[derive(Debug, Clone)]
pub struct AdjacencyMatrix<W> {
    pub(crate) weights: Vec<W>,
    pub(crate) size: usize,
}

impl<W> DistanceMatrix<usize, W> for AdjacencyMatrix<W>
where
    W: core::simd::SimdElement + core::default::Default,
{
    fn get<const L: usize>(&self, pos: Edge<usize, L>) -> (simd::Simd<W, L>, simd::Mask<isize, L>) {
        // 2. Calculate Flat Offsets: index = (row * size) + col
        let size_vec = simd::Simd::<usize, L>::splat(self.size);
        let offsets = (pos.from * size_vec) + pos.to;

        use core::simd::cmp::SimdPartialOrd;
        let mask = offsets
            .simd_lt(core::simd::Simd::splat(self.weights.len()))
            .cast();

        let values = simd::Simd::<W, L>::gather_select(
            &self.weights,
            mask,
            offsets,
            core::simd::Simd::splat(W::default()),
        );

        (values, mask)
    }

    fn set<const L: usize>(
        &mut self,
        pos: Edge<usize, L>,
        value: simd::Simd<W, L>,
        mask: simd::Mask<isize, L>,
    )
    where
        W: core::simd::SimdElement + core::default::Default,
    {
        // Calculate Flat Offsets: index = (row * size) + col
        let size_vec = simd::Simd::<usize, L>::splat(self.size);
        let offsets = (pos.from * size_vec) + pos.to;

        // Bounds check mask
        use core::simd::cmp::SimdPartialOrd;
        let bounds_mask: simd::Mask<isize, L> = offsets
            .simd_lt(simd::Simd::splat(self.weights.len()))
            .cast();

        // Combine the caller's mask with our internal bounds check
        let final_mask = mask & bounds_mask;

        // Perform the Scatter
        // This writes 'value' into 'self.weights' only where final_mask is true
        value.scatter_select(&mut self.weights, final_mask, offsets);
    }

    fn extent(&self) -> usize {
        self.size
    }
}

impl<W> DistanceMatrix<u32, W> for AdjacencyMatrix<W>
where
    W: core::simd::SimdElement + core::default::Default,
{
    fn get<const L: usize>(&self, pos: Edge<u32, L>) -> (simd::Simd<W, L>, simd::Mask<isize, L>) {
        use std::simd::num::SimdUint;
        let casted = Edge {
            from: pos.from.cast::<usize>(),
            to: pos.to.cast::<usize>(),
        };
        <Self as DistanceMatrix<usize, W>>::get(self, casted)
    }

    fn set<const L: usize>(
        &mut self,
        pos: Edge<u32, L>,
        value: simd::Simd<W, L>,
        mask: simd::Mask<isize, L>,
    )
    where
        W: core::simd::SimdElement + core::default::Default,
    {
        use std::simd::num::SimdUint;
        let casted = Edge {
            from: pos.from.cast::<usize>(),
            to: pos.to.cast::<usize>(),
        };
        <Self as DistanceMatrix<usize, W>>::set(self, casted, value, mask)
    }

    fn extent(&self) -> usize {
        <Self as DistanceMatrix<usize, W>>::extent(self)
    }
}

impl<W> DistanceMatrix<u16, W> for AdjacencyMatrix<W>
where
    W: core::simd::SimdElement + core::default::Default,
{
    fn get<const L: usize>(&self, pos: Edge<u16, L>) -> (simd::Simd<W, L>, simd::Mask<isize, L>) {
        use std::simd::num::SimdUint;
        let casted = Edge {
            from: pos.from.cast::<usize>(),
            to: pos.to.cast::<usize>(),
        };
        <Self as DistanceMatrix<usize, W>>::get(self, casted)
    }

    fn set<const L: usize>(
        &mut self,
        pos: Edge<u16, L>,
        value: simd::Simd<W, L>,
        mask: simd::Mask<isize, L>,
    )
    where
        W: core::simd::SimdElement + core::default::Default,
    {
        use std::simd::num::SimdUint;
        let casted = Edge {
            from: pos.from.cast::<usize>(),
            to: pos.to.cast::<usize>(),
        };
        <Self as DistanceMatrix<usize, W>>::set(self, casted, value, mask)
    }

    fn extent(&self) -> usize {
        <Self as DistanceMatrix<usize, W>>::extent(self)
    }
}
