mod etf;

#[derive(Clone, Copy, Debug)]
pub struct KernelArguments {
    pub world: nalgebra::Point2<f32>,
    pub image: nalgebra::Point2<f32>,
}

pub fn coherent<F>(im: &crate::ImageWorldPlacement, mut kernel: F)
where
    F: FnMut(KernelArguments),
{
}

/// Frequency-Modulated Screening
pub fn coherent_do<I, R, F>(
    image: &I,
    image_in_world: &crate::ImageWorldPlacement,
    mut rng: R,
    mut register_func: F,
) where
    I: image::GenericImageView<Pixel: image::Pixel<Subpixel: num_traits::Num>>,
    R: rand::RngCore,
    F: FnMut(nalgebra::Point2<f32>),
{
    // Generate ETF
}
