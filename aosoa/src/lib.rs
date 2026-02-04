#![feature(portable_simd, allocator_api)]

#[doc(hidden)]
pub use paste::paste;

mod bundle;
pub use bundle::SimdBundle;
mod bundle_tuple;
pub use bundle_tuple::SimdBundleTuple;
mod split;
pub use split::SplitSimd;

///
/// # Examples
/// ```
/// #![feature(portable_simd)]
/// struct Point<const L: usize> where core::simd::LaneCount<L>: core::simd::SupportedLaneCount {
///     x: core::simd::Simd<f32, L>,
///     y: core::simd::Simd<f32, L>,
/// }
///
/// impl<const L: usize> aosoa::AsSimdBundleTuple for Point<L>  where core::simd::LaneCount<L>: core::simd::SupportedLaneCount  {
///     type Tuple = (
///         core::simd::Simd<f32, L>,
///         core::simd::Simd<f32, L>
///     );
///
///     #[inline(always)]
///     fn into_tuple(self) -> Self::Tuple {
///         (self.x, self.y)
///     }
///
///     #[inline(always)]
///     fn from_tuple((x, y): Self::Tuple) -> Self {
///         Self {x, y}
///     }
/// }
/// ```
pub trait AsSimdBundleTuple {
    type Tuple;
    fn into_tuple(self) -> Self::Tuple;
    fn from_tuple(t: Self::Tuple) -> Self;
}

pub struct IntoPartsIter<T, const L: usize, const L2: usize>
where
    T: AsSimdBundleTuple<Tuple: SplitSimd<L, L2>>,
{
    chunks: <T::Tuple as SplitSimd<L, L2>>::Iter,
}


pub trait RebindSimd<const L2: usize> {
    type Output;
}

///
/// # Examples
/// Without generics:
/// ```
/// #![feature(portable_simd, allocator_api, trace_macros)]
/// # #[macro_use] extern crate aosoa;
/// # //trace_macros!(true);
/// # fn main() {
/// soa_simd!( Point{
///     x: f32,
///     y: f32,
///     z: f32
/// }, PointSlice, PointAoSoA);
/// # }
/// # //trace_macros!(false);
/// ```
/// With generics:
/// ```
/// #![feature(portable_simd, allocator_api, trace_macros)]
/// # #[macro_use] extern crate aosoa;
/// # //trace_macros!(true);
/// # fn main() {
/// soa_simd!( MyType<T>{
///     x: f32,
///     y: u16,
///     z: T
/// }, MyTypeSlice, MyTypeAoSoa);
/// # }
/// # //trace_macros!(false);
/// ```
#[macro_export]
macro_rules! soa_simd {
    (
        $name:ident $( < $( $generic_type:ident ),* > )? {
            $($field_vis:vis $field:ident: $field_type:ty),+ $(,)?
        },
        $slice_name: ident,
        $aosoa_name: ident $(,)?
     ) => {
        $crate::paste! {
            // The SIMD Type (AoS equivalent for a single Lane)
            #[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
            pub struct $name<$($( $generic_type ),*,)? const L: usize>
            where
                $($( $generic_type : ::core::simd::SimdElement ),*,)?
            {
                $( $field_vis $field: ::core::simd::Simd<$field_type, L>, )+
            }

            // The tuple conversion
            impl<$($( $generic_type ),*,)? const L: usize>
                $crate::AsSimdBundleTuple for $name<$($( $generic_type ),*,)? L>
                where
                    $($( $generic_type : ::core::simd::SimdElement ),*,)?
                {
                type Tuple = (
                    $( ::core::simd::Simd<$field_type, L>, )+
                );

                #[inline(always)]
                fn into_tuple(self) -> Self::Tuple {
                    ($(
                        self.$field
                    ),+,)
                }

                #[inline(always)]
                fn from_tuple(
                    ($($field),*): Self::Tuple
                ) -> Self {
                    //Self {$($field,)+}
                    todo!()
                }
            }
            
            impl<$($( $generic_type ),*,)? const L: usize, const L2: usize>
                $crate::RebindSimd<L2> for $name<$($( $generic_type ),*,)? L>
                where
                    $($( $generic_type : ::core::simd::SimdElement ),*,)?
                {
                type Output = (
                    $( ::core::simd::Simd<$field_type, L>, )+
                );
            }

            // The SoA Storage Type
            #[derive(Clone, Debug, PartialEq, PartialOrd)]
            pub struct $slice_name<$([< $field:upper >]),*> {
                $( $field_vis $field: [< $field:upper >], )*
            }


            impl<$($( $generic_type ),*,)?> $slice_name<$(
                ::std::vec::Vec<$field_type>
            ),*>
            where
                $($( $generic_type : ::core::simd::SimdElement ),*,)?
            {
                pub const fn new() -> Self {
                    Self {
                        $( $field: Vec::new(), )*
                    }
                }

                pub fn with_capacity(capacity: usize) -> Self {
                    Self {
                        $( $field: Vec::with_capacity(capacity), )*
                    }
                }
            }

            // Implementation for Vec storage
            impl<$($( $generic_type ),*,)? A> $slice_name<$(
                ::std::vec::Vec<$field_type, A>
            ),*>
            where
                $($( $generic_type : ::core::simd::SimdElement ),*,)?
                A: ::core::alloc::Allocator + Clone,
            {

                pub fn new_in(alloc: A) -> Self {
                    Self {
                        $( $field: Vec::new_in(alloc.clone()), )*
                    }
                }

                pub fn with_capacity_in(size: usize, alloc: A) -> Self {
                    Self {
                        $( $field: Vec::with_capacity_in(size, alloc.clone()), )*
                    }
                }

                /// Pushes a whole pack
                #[inline(always)]
                pub fn push<const L: usize>(&mut self, item: $name<$($( $generic_type ),*,)? L>)
                    -> ::core::simd::Simd<usize, L>
                where
                {
                    let len = self.len();
                    $( self.$field.extend_from_slice(item.$field.as_array()); )*
                    let new_len = self.len();

                    $crate::iota() + ::core::simd::Simd::splat(new_len - len)
                }

                /// Pushes a whole pack, masked
                /// This is less performant than [Self::push]
                #[inline(always)]
                pub fn push_masked<const L: usize>(&mut self, item: $name<$($( $generic_type ),*,)? L>, mask: ::core::simd::Mask<isize, L>)
                    -> ::core::simd::Simd<usize, L>
                where
                {
                    if mask.all() {
                        return self.push(item);
                    }

                    let len = self.len();
                    for i in 0..L {
                        if mask.test(i) {
                            $( self.$field.push(item.$field[i]); )*
                        }
                    }
                    let new_len = self.len();

                    $crate::iota() + ::core::simd::Simd::splat(new_len - len)
                }

                /// Extends the vec-backed slice with the data of another one
                #[inline(always)]
                pub fn extend<
                    $([< $field:upper >]),*
                >(&mut self, other: &$slice_name<$([< $field:upper >]),*>) {
                    todo!()
                }
            }

            // Implementation for any Slice (Deref)
            impl<$($( $generic_type ,)*)? $([< $field:upper >]),*>
                $slice_name<$([< $field:upper >]),*>
            where
                $($( $generic_type : ::core::simd::SimdElement ),*,)?
                $( [< $field:upper >] : ::core::ops::Deref<Target = [$field_type]> ),*
            {
                #[inline(always)]
                pub fn len(&self) -> usize {
                    // Returns the shortest field length (should be equal)
                    let lengths = [ $( self.$field.len() ),* ];
                    lengths.iter().min().copied().unwrap_or(0)
                }
            }

            // Implementation for any Slice (Deref) with default !
            impl<$($( $generic_type ,)*)? $([< $field:upper >]),*>
                $slice_name<$([< $field:upper >]),*>
            where
                $($( $generic_type : ::core::simd::SimdElement + ::core::default::Default ),*,)?
                $( [< $field:upper >] : ::core::ops::Deref<Target = [$field_type]> ),*
            {
                #[inline(always)]
                pub fn get<const L: usize>(&self, offset: usize) -> (
                    $name<$($( $generic_type ),*,)? L>,
                    ::core::simd::Mask<isize, L>
                )
                where
                {
                    let offsets = ::core::simd::Simd::splat(offset) + $crate::iota();
                    use ::core::simd::cmp::SimdPartialOrd;
                    let mask = offsets.simd_lt(::core::simd::Simd::splat(self.len()));

                    $(
                        let $field = ::core::simd::Simd::load_select_or_default(&self.$field, mask.cast());
                    )*

                    ($name { $($field),* }, mask)
                }

                #[inline(always)]
                pub fn get_gather<const L: usize>(&self, offsets: ::core::simd::Simd<usize, L>) -> (
                    $name<$($( $generic_type ),*,)? L>,
                    ::core::simd::Mask<isize, L>
                )
                where
                {
                    use ::core::simd::cmp::SimdPartialOrd;
                    let mask = offsets.simd_lt(::core::simd::Simd::splat(self.len()));

                    $(
                        let $field = ::core::simd::Simd::gather_select(
                            &self.$field,
                            mask.cast(),
                            offsets,
                            ::core::simd::Simd::splat(::core::default::Default::default())
                        );
                    )*

                    ($name { $($field),* }, mask)
                }

                #[inline(always)]
                pub fn into_iter_simd<'a, const L: usize>(
                    &'a self,
                ) -> impl ::core::iter::Iterator<Item = (
                    $name<$($( $generic_type ),*,)? L>,
                    ::core::simd::Mask<isize, L>
                )> + use<'a, $($( $generic_type ),*,)? $([< $field:upper >]),*, L>
                where
                {
                    let len = self.len();
                    (0..len).step_by(L).map(move |i| self.get::<L>(i))
                }

            }

        }
    };
}

#[doc(hidden)]
#[inline(always)]
pub const fn iota<const L: usize>() -> core::simd::Simd<usize, L>
where
{
    let mut arr = [0; L];

    let mut i = 1;
    while i < L {
        arr[i] = i;
        i += 1;
    }

    core::simd::Simd::from_array(arr)
}

#[derive(Clone, Debug, PartialEq, PartialOrd)]
pub struct Aosoa<C, const L: usize> {
    pub inner: C,
    pub total_len: usize,
}

impl<C, const L: usize> Aosoa<C, L> {
    /// NOT the number of simd vectors: rather the number of elements
    pub fn total_len(&self) -> usize {
        self.total_len
    }
}

// Implementation for AoSoA slice
impl<C, const L: usize> Aosoa<C, L> {
    pub fn iter_simd<'a, Inner>(
        &'a self,
    ) -> impl ::core::iter::Iterator<Item = (Inner, ::core::simd::Mask<isize, L>)> + use<'a, Inner, C, L>
    where
        &'a C: ::core::iter::IntoIterator<Item = &'a Inner>,
        Inner: Clone + 'a,
    {
        let steps = self.total_len.div_ceil(L);
        let remainder = self.total_len % L;
        let full_mask = ::core::simd::Mask::splat(true);
        self.inner
            .into_iter()
            .enumerate()
            .take(steps)
            .map(move |(i, el)| {
                if i + 1 == steps {
                    // The Special Dance:
                    // Calculate how many elements are actually valid in the final lane
                    let mask = if remainder == 0 {
                        full_mask
                    } else {
                        use ::core::simd::cmp::SimdPartialOrd;
                        iota::<L>()
                            .simd_lt(::core::simd::Simd::splat(remainder))
                            .cast()
                    };
                    (el.clone(), mask)
                } else {
                    (el.clone(), full_mask)
                }
            })
    }
}
