#[derive(Clone, Debug)]
pub struct Parameters {
    /// This is in real units
    pub point_size: f32,

    /// Origin of the grid in the world space
    pub origin: nalgebra::Point2<f32>,

    /// How the lines are created.
    /// An angle in radians, trigonometric. (0rad/0deg is horizontal, pi/2rad/90deg is vertical)
    pub orientation: f32,

    /// The distance in real units between two lines.
    /// It should be set to the point_size as a minimum, more for multimaterial prints with angled
    /// screens.
    /// In stochastic screening, this is only a minimum.
    pub interline: f32,

    /// If a point is partly in the image, and partly not, true means excluding it,
    /// false means including it
    pub strict: bool,
}

impl Parameters {
    pub const fn new() -> Self {
        Self {
            point_size: 1.0f32,
            origin: nalgebra::Point2::new(0.0, 0.0),
            orientation: 15.0,
            interline: 4.0,
            strict: true,
        }
    }
}

impl std::default::Default for ScreeningGrid {
    fn default() -> Self {
        Self::new()
    }
}
