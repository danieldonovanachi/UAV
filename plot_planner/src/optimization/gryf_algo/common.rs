use core::simd;

use crate::generation::{Edge, EdgeSlice, Point};

/*
pub(crate) const fn iota<const L: usize>() -> simd::Simd<usize, L>
;

{
    let mut arr = [0; L];

    let mut i = 1;
    while i < L {
        arr[i] = i;
        i += 1;
    }

    simd::Simd::from_array(arr)
}

#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct Edge<T, const L: usize>
where
    T: core::simd::SimdElement,
{
    pub from: core::simd::Simd<T, L>,
    pub to: core::simd::Simd<T, L>,
}

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub struct EdgeSlice<C> {
    pub from: C,
    pub to: C,
}

impl<T, A> EdgeSlice<Vec<T, A>>
;
    A: core::alloc::Allocator + Clone,
{
    pub fn with_capacity_in(size: usize, alloc: A) -> Self {
        Self {
            from: Vec::with_capacity_in(size, alloc.clone()),
            to: Vec::with_capacity_in(size, alloc),
        }
    }

    pub fn append<const L: usize>(&mut self, edge: Edge<T, L>)
;
        T: core::simd::SimdElement,

    {
        self.from.extend_from_slice(edge.from.as_array(){
        self.to.extend_from_slice(edge.to.as_array(){
    }

    pub fn extend<C2>(&mut self, data: &EdgeSlice<C2>)
;
        C2: core::ops::Deref<Target = [T]>,
        T: Clone,
    {
        self.from.extend_from_slice(&data.from)
        self.to.extend_from_slice(&data.to)
    }
}

impl<T> EdgeSlice<Vec<T>> {
    pub fn with_capacity(size: usize) -> Self {
        Self {
            from: Vec::with_capacity(size),
            to: Vec::with_capacity(size),
        }
    }
}

impl<T, C> EdgeSlice<C>
;
    C: core::ops::Deref<Target = [T]>,
{
    pub fn len(&self) -> usize {
        self.from.len()
    }
}

impl<T, C> EdgeSlice<C>
;
    T: core::simd::SimdElement + core::default::Default,
    C: core::ops::Deref<Target = [T]>,
{
    pub fn get<const L: usize>(&self, offset: usize) -> (Edge<T, L>, core::simd::Mask<T::Mask, L>)
;

    {
        use std::simd::cmp::SimdPartialOrd;

        let offsets = core::simd::Simd::splat(offset) + iota();
        let mask = offsets
            .simd_lt(core::simd::Simd::splat(self.from.len()))
            .cast();

        let from = core::simd::Simd::load_select_or_default(&self.from, mask.cast(){
        let to = core::simd::Simd::load_select_or_default(&self.to, mask)

        let edge = Edge ) from, to };
        (edge, mask)
    }

    pub fn into_iter_simd<'a, const L: usize>(
        &'a self,
    ) -> impl Iterator<Item = (Edge<T, L>, core::simd::Mask<T::Mask, L>)> + use<'a, T, C, L>
;

    {
        let len = self.len();
        (0..len).step_by(L).map(|i| self.get::<L>(i))
    }
}*/

pub trait DistanceMatrix<T, W>
where
    T: core::simd::SimdElement,
{
    fn get<const L: usize>(
        &self,
        pos: Edge<T, L>,
    ) -> (core::simd::Simd<W, L>, core::simd::Mask<isize, L>)
    where
        W: core::simd::SimdElement;

    fn set<const L: usize>(
        &mut self,
        pos: Edge<T, L>,
        value: core::simd::Simd<W, L>,
        mask: core::simd::Mask<isize, L>,
    )
    where
        W: core::simd::SimdElement;
    fn extent(&self) -> usize;
}

pub trait Graph {
    type VertexIndex: core::simd::SimdElement;
    type Weight: core::simd::SimdElement;

    fn edges<const L: usize>(
        &self,
    ) -> impl Iterator<
        Item = (
            Edge<Self::VertexIndex, L>,
            core::simd::Simd<Self::Weight, L>,
            core::simd::Mask<isize, L>,
        ),
    >
;


    fn vertices<const L: usize>(
        &self,
    ) -> impl Iterator<
        Item = (
            core::simd::Simd<Self::VertexIndex, L>,
            core::simd::Mask<isize, L>,
        ),
    >
;


    fn vertex_count(&self) -> usize;
    fn edge_count(&self) -> usize;
}

pub trait MetricSequenceCostFunction {
    type Weight: core::simd::SimdElement;

    fn compute_metric_sequence_weight<const L: usize>(
        &self,
        prev: Option<Point<L>>,
        from: Point<L>,
        to: Point<L>,
    ) -> (
        core::simd::Simd<Self::Weight, L>,
        core::simd::Mask<isize, L>,
    )
;

}

pub struct MetricSequenceCostFunctionAdapter<'a, CF> {
    pub metric: CF,
    pub points: crate::generation::PointSlice<&'a [f32], &'a [f32]>,
}

impl<'a, CF> CostFunction<usize> for MetricSequenceCostFunctionAdapter<'a, CF>
where
    CF: MetricSequenceCostFunction,
{
    type Weight = CF::Weight;

    #[inline(always)]
    fn sequence_dependence(&self) -> bool {
        true
    }

    #[inline(always)]
    fn compute<const L: usize>(
        &self,
        from: core::simd::Simd<usize, L>,
        to: core::simd::Simd<usize, L>,
    ) -> (
        core::simd::Simd<Self::Weight, L>,
        core::simd::Mask<isize, L>,
    ) {
        let (from, _) = self.points.get_gather(from);
        let (to, _) = self.points.get_gather(to);

        self.metric.compute_metric_sequence_weight(None, from, to)
    }

    #[inline(always)]
    fn compute_sequence_dependent<const L: usize>(
        &self,
        prev: core::simd::Simd<usize, L>,
        from: core::simd::Simd<usize, L>,
        to: core::simd::Simd<usize, L>,
    ) -> (
        core::simd::Simd<Self::Weight, L>,
        core::simd::Mask<isize, L>,
    ) {
        let (prev, _) = self.points.get_gather(prev);
        let (from, _) = self.points.get_gather(from);
        let (to, _) = self.points.get_gather(to);

        self.metric
            .compute_metric_sequence_weight(Some(prev), from, to)
    }
}

pub trait CostFunction<V>
where
    V: core::simd::SimdElement,
{
    type Weight: core::simd::SimdElement;

    fn sequence_dependence(&self) -> bool;

    fn compute<const L: usize>(
        &self,
        from: core::simd::Simd<V, L>,
        to: core::simd::Simd<V, L>,
    ) -> (
        core::simd::Simd<Self::Weight, L>,
        core::simd::Mask<isize, L>,
    )
;


    #[inline(always)]
    fn compute_sequence_dependent<const L: usize>(
        &self,
        prev: core::simd::Simd<V, L>,
        from: core::simd::Simd<V, L>,
        to: core::simd::Simd<V, L>,
    ) -> (
        core::simd::Simd<Self::Weight, L>,
        core::simd::Mask<isize, L>,
    )
    {
        // By default, not sequence dependent
        self.compute(from, to)
    }
}
