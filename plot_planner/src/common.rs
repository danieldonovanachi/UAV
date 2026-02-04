#[derive(Clone, Debug)]
pub struct ImageWorldPlacement {
    pub im_width: u32,
    pub im_height: u32,

    /// Position, in unit
    pub position: nalgebra::Point2<f32>,
    /// pixels per unit
    pub ppu: f32,
}

impl ImageWorldPlacement {
    pub const fn new(
        im_width: u32,
        im_height: u32,
        position: nalgebra::Point2<f32>,
        ppu: f32,
    ) -> Self {
        Self {
            im_width,
            im_height,
            position,
            ppu,
        }
    }

    /// Size, in units
    pub const fn size(&self) -> nalgebra::Vector2<f32> {
        nalgebra::Vector2::<f32>::new(
            (self.im_width as f32) / self.ppu,
            (self.im_height as f32) / self.ppu,
        )
    }

    pub fn from_image<I>(im: &I, position: nalgebra::Point2<f32>, ppu: f32) -> Self
where
        I: image::GenericImageView,
    {
        Self {
            im_width: im.width(),
            im_height: im.height(),
            position,
            ppu,
        }
    }
}
