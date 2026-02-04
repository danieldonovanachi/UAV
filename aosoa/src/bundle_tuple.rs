use super::SimdBundle;
use core::simd;

pub trait SimdBundleTuple<const L: usize> {
    type ScalarTuple;

    type MaskElement: simd::MaskElement;

    const ARITY: usize;

    fn splat(val: Self::ScalarTuple) -> Self;
}

macro_rules! impl_bundle_tuple {
    // The base case: Single bundle (not a tuple)
    ($head:ident) => {
        impl<$head, const L: usize> SimdBundleTuple<L> for ($head,)
        where
            $head: SimdBundle<L>,
        {
            type ScalarTuple = ($head::Scalar,);
            type MaskElement = $head::MaskElement;

            const ARITY: usize = 1;

            #[inline(always)]
            fn splat(val: Self::ScalarTuple) -> Self {
                ($head::splat(val.0),)
            }
        }

        impl<$head, const L: usize> SimdBundleTuple<L> for $head
        where
            $head: SimdBundle<L>,
        {
            type ScalarTuple = $head::Scalar;
            type MaskElement = $head::MaskElement;

            const ARITY: usize = 1;

            #[inline(always)]
            fn splat(val: Self::ScalarTuple) -> Self {
                $head::splat(val)
            }
        }
    };

    // The recursive/tuple case
    ($($ts:ident),*) => {
        impl<$($ts),+, const L: usize> SimdBundleTuple<L> for ($($ts),+)
        where
            $($ts: SimdBundle<L>),+,
        {
            type ScalarTuple = ($($ts::Scalar),+);

            // For multi-input masks, we typically default to isize
            // to stay compatible with standard SIMD mask layouts.
            // They are all inter-castable anyway.
            type MaskElement = isize;

            const ARITY: usize = <[()]>::len(&[ $(impl_bundle_tuple!(@substitute $ts)),* ]);

            #[inline(always)]
            fn splat(val: Self::ScalarTuple) -> Self {
                // Destructure the scalar tuple and splat each part into the bundle tuple
                #[allow(non_snake_case)]
                let ($($ts,)+) = val;
                ($($ts::splat($ts)),+)
            }
        }
    };

    // Helper to count tokens for arity
    (@substitute $_t:ident) => { () };
}

impl_bundle_tuple!(B1);
impl_bundle_tuple!(B1, B2);
impl_bundle_tuple!(B1, B2, B3);
impl_bundle_tuple!(B1, B2, B3, B4);
impl_bundle_tuple!(B1, B2, B3, B4, B5);
impl_bundle_tuple!(B1, B2, B3, B4, B5, B6);
impl_bundle_tuple!(B1, B2, B3, B4, B5, B6, B7);
impl_bundle_tuple!(B1, B2, B3, B4, B5, B6, B7, B8);
