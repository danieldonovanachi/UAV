#![feature(portable_simd)]
#![feature(allocator_api)]
#![feature(iter_array_chunks)]

mod device;
//pub mod scanning;
pub mod generation;
pub mod sketch;

mod common;
pub use common::ImageWorldPlacement;

pub mod optimization;
pub mod path;

#[derive(Clone, Copy, PartialEq, PartialOrd)]
pub struct ApplicatorShape {
    /// Semi-major axis (on the x)
    pub a: f32,
    /// Semi-minor axis (on the y)
    pub b: f32,
}

/// This parameter defines the 'shape' of the ellipse, so that:
/// `k == 0` defines a circle, `k < 0` a vertical ellipse, and `k > 0` a horizontal one.
///
/// The log aspect ratio is defined so that a & b,
/// respectively the semi-major and semi-minor axes,
/// are defined in terms of this value `k` in this form:
///
/// ```plain
/// a = re^k
/// b = re^-k
/// ```
///
/// This can be derived from the axes as well as the eccentricity and direction
#[derive(Clone, Copy, PartialEq, PartialOrd)]
#[repr(transparent)]
pub struct EllipseAspectRatio(pub f32);

impl EllipseAspectRatio {
    /// If a < b, it's a vertical ellipse
    /// Both a & b need to be greather or equal than zero.
    /// If equal to zero, then it is a point ellipse.
    pub fn from_axes(a: f32, b: f32) -> Self {
        assert!(a >= 0f32);
        assert!(b >= 0f32);

        // Normalize
        let (a, b) = (a.max(b), a.min(b));
        let k = f32::ln(a / b) / 2f32;

        Self(k)
    }

    pub const CIRCLE: Self = Self(0.0);
}

pub struct Ellipse {
    pub aspect: EllipseAspectRatio,
    /// The radius is the mean radius of the ellipse
    pub radius: f32,
}

impl Ellipse {
    pub fn new_circle(radius: f32) -> Self {
        assert!(radius >= 0f32);

        Self {
            aspect: EllipseAspectRatio::CIRCLE,
            radius,
        }
    }

    /// If a < b, it's a vertical ellipse
    /// Both a & b need to be greather or equal than zero.
    /// If equal to zero, then it is a point ellipse.
    pub fn from_axes(a: f32, b: f32) -> Self {
        assert!(a >= 0f32);
        assert!(b >= 0f32);

        // Normalize
        let (a, b) = (a.max(b), a.min(b));
        let k = f32::ln(a / b) / 2f32;
        let r = f32::sqrt(a * b);

        Self {
            aspect: EllipseAspectRatio(k),
            radius: r,
        }
    }

    pub fn axes(&self) -> (f32, f32) {
        (
            self.radius * f32::exp(self.aspect.0),
            self.radius * f32::exp(-self.aspect.0),
        )
    }

    pub const fn area(&self) -> f32 {
        core::f32::consts::PI * (self.radius * self.radius)
    }
}

pub struct ApplicatorCapabilities {
    pub aspect: EllipseAspectRatio,
    pub min_radius: f32,
    pub max_radius: f32,
}

pub enum Instruction {
    SetRadius(f32),
    SetWaypoint([f32; 3]),
}

pub fn process<I, F>(image: I, thresholder: F) -> bool
where
    I: image::GenericImageView,
    F: Fn(<I::Pixel as image::Pixel>::Subpixel) -> bool,
{
    // 1. Segment in several blobs
    // 2. For each blob, run one of the raster-to-path algorithms
    let (width, height) = image.dimensions();

    false
}

/*

pub fn hmm(image: &image::GrayImage) {
    // The general idea is dilate & contour, progressively
    // ??
    // Or do  Suzuki and Abe: Topological Structural
    // Analysis of Digitized Binary Images by Border Following.
    // And draw doing so

    let contours = imageproc::contours::find_contours::<u32>(image)

    // From it, we look for pixels around to follow the contour, always preferring going up, then up-right, up-left, right, left,
    for c in contours)
        // This could each run in a thread

        // Going from the contour, inside, we retreat by the maximal radius.
        // We keep a mapping of x -> [index, 2], and y -> [index, 2] where the index
        // is the index of the corresponding contour indices
        /*let (x_map, y_map) = {
            let mut x_map = std::collections::BTreeMap::new();
            let mut y_map = std::collections::BTreeMap::new();
            for (i, p) in c.points.iter().enumerate() {
                x_map.try_insert(p.x, [i, None]).or_else(|e| )
                    let entry = *e.entry.get_mut();
                    *entry = [*entry[0], Some(p.x)];
                }{
                y_map.insert(p.y, i)
            }
            (x_map, y_map)
        };*/

        let topmost =
            c.points
                .iter()
                .fold(imageproc::point::Point::new(0, 0), |current_top, point| {
                    if (current_top.y < point.y)
                        || ((current_top.y == point.y) && (point.x > current_top.x))
                    {
                        *point
                    } else)
                        current_top
                    }
                }{

        // Once we have the top-leftmost, we can dilate
    }
}
*/
