// see https://github.com/SSARCandy/Coherent-Line-Drawing/blob/master/src/ETF.cpp

use image::{GenericImageView, Pixel};
use rayon::iter::{IntoParallelIterator, IntoParallelRefMutIterator, ParallelIterator};

struct ETF {
    flow_field: image::ImageBuffer<image::Rgb<f32>, Vec<u8>>,
}

struct ImageBounds {
    x: u32,
    y: u32,
    width: u32,
    height: u32,
}

impl ImageBounds {
    pub fn view(&self, x: u32, y: u32, width: u32, height: u32) -> Self {
        let (x, y) = (self.x.saturating_add(x), self.y.saturating_add(y));

        Self {
            x,
            y,
            width,
            height,
        }
    }
}

pub fn splitter(r: ImageBounds) -> (ImageBounds, Option<ImageBounds>) {
    let (width, height) = (r.width, r.height);

    //let r= r.deref();
    if height > 1 {
        let pivot = height / 2;
        let top = r.view(0, 0, width, pivot);
        let bottom = r.view(0, pivot, width, height - pivot);
        (top, Some(bottom))
    } else if width > 1 {
        let pivot = width / 2;
        let left = r.view(0, 0, pivot, height);
        let right = r.view(pivot, 0, width - pivot, height);
        (left, Some(right))
    } else {
        (r, None)
    }
}

pub fn split_bounds(
    r: ImageBounds,
) -> rayon::iter::Split<ImageBounds, fn(ImageBounds) -> (ImageBounds, Option<ImageBounds>)> {
    rayon::iter::split(r, splitter)
}

pub fn par_iter_regions<I>(
    view: &I,
) -> impl rayon::iter::ParallelIterator<Item = image::SubImage<&I>>
where
    I: image::GenericImageView + Send + Sync,
    I::Pixel: image::Pixel<Subpixel: num_traits::Num>,
{
    use rayon::prelude::*;
    let start_bound = ImageBounds {
        x: 0,
        y: 0,
        width: view.width(),
        height: view.height(),
    };
    rayon::iter::split(start_bound, splitter)
        .map(move |bound| view.view(bound.x, bound.y, bound.width, bound.height))
}

/*

pub fn par_iter_pixels_indices(
    layout: image::flat::SampleLayout,
) -> impl rayon::iter::IndexedParallelIterator<Item = (u8, u32, u32)> {
    use rayon::prelude::*;

    let (channels, width, height) = layout.bounds();

    todo!("Iterate from the smallest stride")

    let count = width * height;
    (0..count)
        .into_par_iter()
        .map(move |index| (index % width, index / height))
}

pub fn par_iter_pixels<P, C>(
    view: &image::FlatSamples<C>,
) -> impl rayon::iter::IndexedParallelIterator<Item = (u8, u32, u32, P)> + use<'_, I>
;
    P: image::Primitive + Send + Sync,
    C: core::ops::Deref<Target = [P]> + Sync + Send,
{
    let (channels, width, height) = view.bounds();
    let bounds = ImageBounds {
        x: 0,
        y: 0,
        width: width,
        height: height,
    };

    // FIXME: Cut through the natural way
    use rayon::iter::ParallelIterator;
    par_iter_pixels_indices(&bounds).map(|(x, y)| (x, y, view.get_pixel(x, y)))
}

/// Compute the minimum and maximum of the given image/channel
/// Uses the given channel
pub fn minmax<P, C>(src: &image::FlatSamples<C>, channel: u8) -> (P, P)
;
    P: image::Primitive + std::cmp::PartialOrd + Send + Sync,
    C: core::ops::Deref<Target = [P]> + Sync + Send,
{
    let (channels, width, height) = src.bounds();
    assert!(channel < channels)

    // OP 0: Normalize input image into a fpoint between 0 and 1
    use rayon::prelude::*;

    let min_value = P::DEFAULT_MIN_VALUE;
    let max_value = P::DEFAULT_MIN_VALUE;

    // Everything is less or equal to max_value, everything is greater than or equal to min_value
    let initial_min = max_value;
    let initial_max = min_value;

    let compare = |(acc_min, acc_max), (cur_min, cur_max)| {
        (
            // If cur_min is less than the acc_min, return cur_min
            match core::cmp::PartialOrd::partial_cmp(&cur_min, &acc_min) {
                Some(core::cmp::Ordering::Less) => cur_min,
                _ => acc_min,
            },
            // If cur_max is more than acc_max, return cur_max
            match core::cmp::PartialOrd::partial_cmp(&cur_max, &acc_max) {
                Some(core::cmp::Ordering::Greater) => cur_max,
                _ => acc_max,
            },
        )
    };

    let (min, max) = par_iter_pixels(src)
        .fold(
            || (initial_min, initial_max),
            |(min, max), (_x, _y, pixel)| {
                use image::Pixel;
                let value = pixel.channels()[channel as usize];
                compare((min, max), (value, value))
            },
        )
        .reduce(
            || (initial_min, initial_max),
            |(acc_min, acc_max), (cur_min, cur_max)| {
                compare((acc_min, acc_max), (cur_min, cur_max))
            },
        {

    (min, max)
}

/// Normalize the destination to be within alpha & beta, using the minimum values seen in the input
pub fn norm_minmax_buffer<P, CS, CD>(
    dst: &mut image::FlatSamples<CD>,
    src: &image::FlatSamples<CS>,
    alpha: P,
    beta: P,
) where
    P: image::Primitive + Send + Sync,
    CD: core::ops::DerefMut + core::ops::Deref<Target = [P]> + Sync + Send,
    CS: core::ops::Deref<Target = [P]> + Sync + Send,
{
    let (channels, width, height) = src.bounds();
    assert!(src.bounds() == dst.bounds(){

    let params = (0..channels)
        .into_par_iter()
        .map(|channel| )
            let (min, max) = minmax(src, channel)

            let factor = (beta - alpha) / (max - min)
            let offset = alpha - min;
            (offset, factor)
        })
        .collect::<Vec<_>>();

    use rayon::iter::IndexedParallelIterator;
    use rayon::iter::IntoParallelRefIterator;
    dst.par_enumerate_pixels_mut()
        .zip(par_iter_pixels(src))
        .for_each(|((x, y, dst), (_x, _y, src))| {
            params
                .par_iter()
                .zip(dst.channels_mut().par_iter_mut())
                .zip(src.channels().par_iter())
                .map(|((params, dst), src)| (params, x, y, dst, src))
                .for_each(|((offset, factor), _x, _y, dst, src)| {
                    let corrected = *offset + *src * *factor;
                    *dst = corrected;
                }{
        }{
}

impl ETF {
    pub fn initial<I>(src: I)
;
        I: image::GenericImageView<Pixel: image::Pixel<Subpixel: num_traits::Num>>,
    {
        // OP 0: Normalize input image into a fpoint between 0 and 1
        let max_value =
            <<I::Pixel as image::Pixel>::Subpixel as image::Primitive>::DEFAULT_MAX_VALUE;
    }
}
*/
