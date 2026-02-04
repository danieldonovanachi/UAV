extern crate alloc;
use crate::generation::{Dot, DotSlice, Edge, EdgeSlice, GenerationBuffer, Point, PointSlice};

pub struct GenerationBufferVec<A: core::alloc::Allocator> {
    pub points: PointSlice<Vec<f32, A>, Vec<f32, A>>,
    pub edges: EdgeSlice<Vec<usize, A>, Vec<usize, A>>,
    pub dots: DotSlice<Vec<usize, A>>,
}

impl GenerationBufferVec<alloc::alloc::Global> {
    pub const fn new() -> Self {
        Self {
            points: PointSlice::new(),
            edges: EdgeSlice::new(),
            dots: DotSlice::new(),
        }
    }
}

impl<A> GenerationBuffer for GenerationBufferVec<A>
where
    A: core::alloc::Allocator + Clone,
{
    fn push_points<const L: usize>(
        &mut self,
        point: Point<L>,
        mask: core::simd::Mask<isize, L>,
    ) -> core::simd::Simd<usize, L> {
        self.points.push_masked(point, mask)
    }

    fn push_lines<const L: usize>(
        &mut self,
        line: Edge<usize, L>,
        mask: core::simd::Mask<isize, L>,
    ) -> core::simd::Simd<usize, L> {
        self.edges.push_masked(line, mask)
    }

    fn push_dots<const L: usize>(
        &mut self,
        dots: Dot<L>,
        mask: core::simd::Mask<isize, L>,
    ) -> core::simd::Simd<usize, L> {
        self.dots.push_masked(dots, mask)
    }
}
