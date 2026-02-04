mod common;
pub use common::{
    Dot, DotSlice, Edge, EdgeSlice, GenerationBuffer, GenerationProcess, Generator, Point,
    PointSlice,
};

mod screening;
pub use screening::ScreeningGrid;
#[cfg(feature = "hdp")]
pub use screening::{FMScreeningGenerator, FMScreeningProcess};

mod vec_generation_buffer;
pub use vec_generation_buffer::GenerationBufferVec;

#[cfg(feature = "hdp")]
pub use hdp_iter::common as hdp_common;

#[cfg(not(feature = "hdp"))]
pub mod hdp_common {
    // Stub module when hdp_iter is not available
    pub mod memory {
        use image::GenericImage;
        use core::simd;
        
        pub struct Cube<I> {
            _phantom: std::marker::PhantomData<I>,
        }
        
        impl<I> Cube<I> {
            pub fn from_image(_image: I) -> Self {
                Self {
                    _phantom: std::marker::PhantomData,
                }
            }
        }
        
        impl<I> AsRef<I> for Cube<I> {
            fn as_ref(&self) -> &I {
                // This is a stub - won't actually work
                panic!("hdp_iter feature not enabled. Please enable 'hdp' feature or provide hdp_iter dependency.")
            }
        }
        
        pub mod utils {
            use core::simd;
            
            pub struct PositionDecimal<const L: usize> {
                pub x: simd::Simd<f32, L>,
                pub y: simd::Simd<f32, L>,
                pub c: simd::Simd<f32, L>,
            }
        }
    }
}
