use aosoa::soa_simd;
use core::simd;

soa_simd!(
    Point {
        pub x: f32,
        pub y: f32
    },
    PointSlice,
    PointAoSoA
);

impl<const L: usize> From<nalgebra::Vector2<simba::simd::Simd<simd::Simd<f32, L>>>> for Point<L>
{
    fn from(value: nalgebra::Vector2<simba::simd::Simd<simd::Simd<f32, L>>>) -> Self {
        Self {
            x: value.x.0,
            y: value.y.0,
        }
    }
}

impl<const L: usize> From<nalgebra::Point2<simba::simd::Simd<simd::Simd<f32, L>>>> for Point<L>
 {
    fn from(value: nalgebra::Point2<simba::simd::Simd<simd::Simd<f32, L>>>) -> Self {
        Self {
            x: value.x.0,
            y: value.y.0,
        }
    }
}

impl<const L: usize> Into<nalgebra::Point2<simba::simd::Simd<simd::Simd<f32, L>>>> for Point<L>
 {
    fn into(self) -> nalgebra::Point2<simba::simd::Simd<simd::Simd<f32, L>>> {
        nalgebra::Point2::new(simba::simd::Simd(self.x), simba::simd::Simd(self.y))
    }
}

impl<const L: usize> core::ops::Add for Point<L>
 {
    type Output = Self;

    #[inline(always)]
    fn add(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl<const L: usize> core::ops::Sub for Point<L>
 {
    type Output = Self;

    #[inline(always)]
    fn sub(self, rhs: Self) -> Self::Output {
        Self {
            x: self.x - rhs.x,
            y: self.y - rhs.y,
        }
    }
}

impl<const L: usize> core::ops::AddAssign for Point<L>
 {
    #[inline(always)]
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

impl<const L: usize> core::ops::SubAssign for Point<L>
 {
    #[inline(always)]
    fn sub_assign(&mut self, rhs: Self) {
        self.x -= rhs.x;
        self.y -= rhs.y;
    }
}

soa_simd!(
    Edge<T> {
        pub from: T,
        pub to: T
    },
    EdgeSlice,
    EdgeAoSoA
);

soa_simd!(
    Dot {
        pub index: usize
    },
    DotSlice,
    DotAoSoA
);

pub trait GenerationBuffer {
    fn push_points<const L: usize>(
        &mut self,
        point: Point<L>,
        mask: simd::Mask<isize, L>,
    ) -> simd::Simd<usize, L>;

    fn push_lines<const L: usize>(
        &mut self,
        line: Edge<usize, L>,
        mask: simd::Mask<isize, L>,
    ) -> simd::Simd<usize, L>;

    fn push_dots<const L: usize>(
        &mut self,
        point_indices: Dot<L>,
        mask: simd::Mask<isize, L>,
    ) -> simd::Simd<usize, L>;
}

pub enum GenerationControlFlow<E> {
    Finished,
    Ongoing {
        /// Units of work completed in this specific call
        delta: usize,
    },
    Error(E),
}

pub trait GenerationProcess<S> {
    type Error;

    fn generate<B: GenerationBuffer>(
        &mut self,
        image: &crate::generation::hdp_common::memory::Cube<&[S]>,
        buffer: &mut B,
        count: usize,
    ) -> GenerationControlFlow<Self::Error>;

    // Steps left, minimum & maximum bound
    fn min_left(&self) -> (usize, Option<usize>);
}

pub trait Generator<S> {
    type Config: Clone;
    type Process: GenerationProcess<S>;

    fn start(image: &crate::ImageWorldPlacement, config: Self::Config) -> Self::Process;
}
