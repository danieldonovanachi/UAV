use image::Pixel;

pub mod consts;

mod grid;
pub use grid::ScreeningGrid;

mod common;
use common::{prepare_screen, ScreeningBounds};

mod iterator;

use core::simd;
use std::simd::MaskElement;

#[cfg(feature = "hdp")]
pub struct FMScreeningProcess<R> {
    rng: R,
    inner: iterator::ScreeningIterator<16>,
}

#[cfg(feature = "hdp")]
impl<S, R, CmpMask> crate::generation::GenerationProcess<S> for FMScreeningProcess<R>
where
    R: rand::RngCore,
    S: core::simd::SimdElement
        + core::default::Default
        + rand::distr::uniform::SampleUniform
        + num_traits::Float,
    simd::Simd<S, 16>: simd::cmp::SimdPartialOrd<Mask = core::simd::Mask<CmpMask, 16>>,
    CmpMask: simd::MaskElement,
{
    type Error = ();

    fn generate<B: super::GenerationBuffer>(
        &mut self,
        image: &crate::generation::hdp_common::memory::Cube<&[S]>,
        buffer: &mut B,
        count: usize,
    ) -> super::common::GenerationControlFlow<Self::Error> {
        const L: usize = 16;

        core::iter::Iterator::take(&mut self.inner, count.div_ceil(L)).for_each(
            |(kernel_args, mask)| {
                #[cfg(feature = "hdp")]
                let sampled = {
                    use hdp_iter::common::memory::utils::PositionDecimal;
                    image.sample_nearest(PositionDecimal { x: kernel_args.image.x,
                        y: kernel_args.image.y,
                        c: simd::Simd::splat(0.0),
                    })
                };
                #[cfg(not(feature = "hdp"))]
                let sampled = simd::Simd::splat(S::zero());

                use core::simd::cmp::SimdPartialOrd;
                use rand::Rng;
                let threshold: simd::Simd<S, L> =
                    simd::Simd::from_array(core::array::from_fn(|_| {
                        // FIXME
                        // This does NOT work with HDR, where all the bright areas will be 1 or above.
                        // Thus, we need to work with a monotonic mapping of HDR -> SDR.
                        // This problem is shared with all generators though.
                        self.rng.random_range(S::zero()..=S::one())
                    }));

                let mask = mask & sampled.simd_ge(threshold).cast();

                let indices = buffer.push_points(kernel_args.world, mask);
                let dot = crate::generation::common::Dot { index: indices };
                buffer.push_dots(dot, mask);
            }
        );

        super::common::GenerationControlFlow::Ongoing { delta: count }
    }

    fn min_left(&self) -> (usize, Option<usize>) {
        todo!()
    }
}

#[cfg(feature = "hdp")]
pub struct FMScreeningGenerator<R>(core::marker::PhantomData<R>);

#[cfg(feature = "hdp")]
impl<R> FMScreeningGenerator<R> {
    pub const fn new() -> Self {
        Self(core::marker::PhantomData)
    }
}

#[cfg(feature = "hdp")]
impl<S, R, CmpMask> crate::generation::Generator<S> for FMScreeningGenerator<R>
where
    R: rand::RngCore + Clone,
    S: core::simd::SimdElement
        + core::default::Default
        + rand::distr::uniform::SampleUniform
        + num_traits::Float,
    simd::Simd<S, 16>: simd::cmp::SimdPartialOrd<Mask = core::simd::Mask<CmpMask, 16>>,
    CmpMask: simd::MaskElement,
{
    type Config = (ScreeningGrid, R);
    type Process = FMScreeningProcess<R>;

    fn start(image: &crate::ImageWorldPlacement, config: Self::Config) -> Self::Process {
        FMScreeningProcess {
            rng: config.1,
            inner: iterator::ScreeningIterator::new(image, config.0),
        }
    }
}
