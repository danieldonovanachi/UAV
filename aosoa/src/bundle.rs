use core::simd;

pub trait SimdBundle<const L: usize> {
    type Scalar;
    type MaskElement: simd::MaskElement;

    // This allows the pipeline to create "Zero" or "Identity" bundles
    fn splat(val: Self::Scalar) -> Self;

    fn as_array(&self) -> &[Self::Scalar; L];
}

impl<T, const L: usize> SimdBundle<L> for simd::Simd<T, L>
where
    T: simd::SimdElement,
{
    type Scalar = T;
    type MaskElement = T::Mask;

    fn splat(val: T) -> Self {
        simd::Simd::splat(val)
    }

    fn as_array(&self) -> &[Self::Scalar; L] {
        self.as_array()
    }
}

