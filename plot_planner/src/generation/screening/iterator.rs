use super::{prepare_screen, ScreeningBounds, ScreeningGrid};
use core::simd;
use image::Pixel;
use simba::scalar::SupersetOf;

#[derive(Clone, Copy, Debug)]
pub struct KernelArguments<const L: usize> {
    pub world: crate::generation::Point<L>,
    pub image: crate::generation::Point<L>,
}

#[cfg(feature = "hdp")]
#[derive(Clone, Debug)]
pub struct ScreeningIterator<const L: usize> {
    inner: hdp_iter::CoordinatesIterator<L>,
    bounds: ScreeningBounds,
    grid: ScreeningGrid,
}

#[cfg(feature = "hdp")]
impl<const L: usize> ScreeningIterator<L> {
    pub fn new(im: &crate::ImageWorldPlacement, grid: ScreeningGrid) -> Self {
        let bounds = prepare_screen(im, &grid);

        // We voluntarily do not iterate on the image, but on the calculated bounds
        // of the grid that lay within our zone of interest.
        let inner = hdp_iter::CoordinatesIterator::new(
            hdp_iter::common::Bounds { width: (bounds.i_range[1] - bounds.i_range[0]) as usize,
                height: (bounds.j_range[1] - bounds.j_range[0]) as usize,
                depth: 1, /* channels */
            },
            hdp_iter::common::IterationOrder::ZXY,
            hdp_iter::common::Position::<1>::new_scalar(
                bounds.i_range[0] as usize,
                bounds.j_range[0] as usize,
                0,
            ),
        );

        Self {
            inner,
            bounds,
            grid,
        }
    }
}

#[cfg(feature = "hdp")]
impl<const L: usize> Iterator for ScreeningIterator<L>
where
    simba::simd::Simd<simd::Simd<f32, L>>: simba::simd::SimdRealField<Element = f32>,
{
    type Item = (KernelArguments<L>, simd::Mask<isize, L>);

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        while let Some((position, mask)) = self.inner.next() {
            // Compute Local Grid Coordinates (P')
            // P' = (i * Pg, j * Pg)
            let resolution = simd::Simd::splat(self.grid.resolution);

            use std::simd::num::SimdUint;
            let local_x = position.x.cast() * resolution;
            let local_y = position.y.cast() * resolution;
            let p_local = nalgebra::Point2::<simba::simd::Simd<simd::Simd<f32, L>>>::new(
                simba::simd::Simd(local_x),
                simba::simd::Simd(local_y),
            );

            // Transform P' back to World Space (P)
            // P = O_g + R(Theta_g) * P'
            let isometry = self.bounds.grid_to_world;
            // We need to unpack/repack only because the `Unit` is a pain in the ass for casting
            let isometry = nalgebra::Isometry {
                rotation: nalgebra::Unit::new_unchecked(nalgebra::Complex { re: simba::simd::Simd(simd::Simd::splat(isometry.rotation.re)),
                    im: simba::simd::Simd(simd::Simd::splat(isometry.rotation.im)),
                }),
                translation: isometry.translation.cast(),
            };
            let p_world = isometry.transform_point(&p_local);

            // World-space Containment Check

            use simd::cmp::SimdPartialOrd;
            let is_inside_x = {
                let over = p_world
                    .x
                    .0
                    .simd_ge(simd::Simd::splat(self.bounds.im_x_range[0]));
                let under = p_world
                    .x
                    .0
                    .simd_le(simd::Simd::splat(self.bounds.im_x_range[1]));
                over & under
            };

            let is_inside_y = {
                let over = p_world
                    .y
                    .0
                    .simd_ge(simd::Simd::splat(self.bounds.im_y_range[0]));
                let under = p_world
                    .y
                    .0
                    .simd_le(simd::Simd::splat(self.bounds.im_y_range[1]));
                over & under
            };

            let is_inside = is_inside_x & is_inside_y;
            let mask = mask.cast() & is_inside;
            if !mask.any() {
                continue;
            }

            let image_position = nalgebra::Point2::new(
                simba::simd::Simd(simd::Simd::splat(self.bounds.im_x_range[0])),
                simba::simd::Simd(simd::Simd::splat(self.bounds.im_y_range[0])),
            );

            let image_end = nalgebra::Point2::new(
                simba::simd::Simd(simd::Simd::splat(self.bounds.im_x_range[1])),
                simba::simd::Simd(simd::Simd::splat(self.bounds.im_y_range[1])),
            );

            let image_size = image_end - image_position;

            let pixel_coords_f = (p_world - image_position).zip_map(&image_size, |a, b| a / b);

            return Some((
                KernelArguments { world: p_world.into(),
                    image: pixel_coords_f.into(),
                },
                mask.cast(),
            ));
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use imageproc::definitions::HasWhite;
    use rand::SeedableRng;

    use crate::path::SectionBuffer;

    const MAIN_IMAGE_FILENAME: &'static str = "Fionn.jpg";

    static TESTDATA_DIR_PATH: std::sync::LazyLock<std::path::PathBuf> =
        std::sync::LazyLock::new(|| {
            let mut path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
            path.push("testdata");
            path
        });

    fn buffer_to_images(
        buf: &SectionBuffer,
        placement: crate::ImageWorldPlacement,
        order: &[usize],
    ) {
        let mut image =
            image::ImageBuffer::<image::Luma<u8>, _>::new(placement.im_width, placement.im_height);

        for (i, point) in order.iter().map(|k| buf.points[*k]).enumerate() {
            let pixel = ((point.position - placement.position) * placement.ppu).map(|v| v as i32);
            let radius = (point.size * placement.ppu / 2.0) as i32;

            imageproc::drawing::draw_filled_circle_mut(
                &mut image,
                (pixel.x, pixel.y),
                radius,
                image::Luma::white(),
            );

            let name = format!("output_{:0>4}.png", i);
            println!("[{}/{}] Saving to <{}>", i, buf.points.len(), &name);
            image.save(&name).unwrap();

            // Draw it again in gray
            imageproc::drawing::draw_filled_circle_mut(
                &mut image,
                (pixel.x, pixel.y),
                radius,
                image::Luma([u8::MAX / 4]),
            );
        }

        {
            let mut image = image::ImageBuffer::<image::Luma<u8>, _>::new(
                placement.im_width,
                placement.im_height,
            );

            for (i, point) in buf.points.iter().enumerate() {
                let pixel =
                    ((point.position - placement.position) * placement.ppu).map(|v| v as i32);
                let radius = (point.size * placement.ppu / 2.0) as i32;

                imageproc::drawing::draw_filled_circle_mut(
                    &mut image,
                    (pixel.x, pixel.y),
                    radius,
                    image::Luma::white(),
                );
            }
            image.save("final.png").unwrap();
        }
    }

    #[test]
    pub fn screen_fm() {
        let image_path = TESTDATA_DIR_PATH.join(MAIN_IMAGE_FILENAME);
        let reader = image::ImageReader::open(&image_path)
            .expect(&format!("image loading failed: {:?}", &image_path));
        let decoded = reader.decode().expect("image decoding failed");

        // Now, we grayscale it
        let grayscale = decoded.to_luma8();

        const PPU: f32 = 2.0;
        let image_in_world =
            crate::ImageWorldPlacement::from_image(&grayscale, nalgebra::Point2::default(), PPU);
        let grid = super::ScreeningGrid {
            resolution: 1.0,
            point_size: 1.5,
            ..std::default::Default::default()
        };
        let mut rng = rand_xoshiro::Xoroshiro64Star::seed_from_u64(0);

        let mut path_recorder = crate::path::SectionBuffer::new();
        super::screen_fm(&grayscale, &image_in_world, &grid, &mut rng, |p| {
            path_recorder.push_point(crate::path::Point::new(p, grid.point_size));
        });

        let image_output_in_world = {
            let mut source = image_in_world;
            /*source.im_width /= 2;
            source.im_height /= 2;
            source.ppu = 4.0;*/
            source
        };
        //

        // OPTIMIZE

        let characteristics = crate::optimization::SpecificEnergyCost {
            up: 1.5,
            down: 0.1,
            forward: 1.0,
        };
        let optimizer = crate::optimization::OptimizationSettings {
            specific_energy: characteristics,
            start: nalgebra::Point2::new(0.0, 0.0),
            include_start: true,
        };

        let order = optimizer.optimize_points(&path_recorder.points);

        buffer_to_images(&path_recorder, image_output_in_world, order.as_slice());
    }
}
