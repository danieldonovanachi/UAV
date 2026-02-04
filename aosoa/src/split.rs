use core::simd;

pub struct SimdChunkIter<T, const L: usize, const L2: usize> {
    data: [T; L],
    index: usize,
}


impl<T, const L: usize, const L2: usize> SimdChunkIter<T, L, L2>
where
    T: simd::SimdElement
{
    fn new(simd: simd::Simd<T, L>) -> Self {
        Self {
            data: simd.to_array(),
            index: 0,
        }
    }
}

impl<T, const L: usize, const L2: usize> Iterator for SimdChunkIter<T, L, L2>
where
    T: simd::SimdElement,
{
    type Item = simd::Simd<T, L2>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index + L2 > L {
            return None;
        }

        let chunk = simd::Simd::from_array(
            self.data[self.index..self.index + L2]
                .try_into()
                .ok()?,
        );

        self.index += L2;
        Some(chunk)
    }
}

pub trait SplitSimd<const L: usize, const L2: usize> {
    type Item;
    type Iter: Iterator<Item = Self::Item>;

    fn split(self) -> Self::Iter;
}

impl<T, const L: usize, const L2: usize> SplitSimd<L, L2> for simd::Simd<T, L>
where
    T: simd::SimdElement,
{
    type Item = simd::Simd<T, L2>;
    type Iter = SimdChunkIter<T, L, L2>;

    fn split(self) -> Self::Iter {
        SimdChunkIter::new(self)
    }
}
