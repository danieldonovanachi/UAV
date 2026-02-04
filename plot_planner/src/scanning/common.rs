/// Holds all pre-calculated values required for the grid iteration loop.
#[derive(Debug)]
pub(super) struct ScreeningBounds {
    /// Isometry to transform local grid coords (i*Pg, j*Pg) back to world space.
    pub grid_to_world: nalgebra::Isometry2<f32>,
    /// [min, max] grid index along the X (i) axis.
    pub i_range: [i64; 2],
    /// [min, max] grid index along the Y (j) axis.
    pub j_range: [i64; 2],
    /// [min, max] world X coordinate of the image rectangle for final check.
    pub im_x_range: [f32; 2],
    /// [min, max] world Y coordinate of the image rectangle for final check.
    pub im_y_range: [f32; 2],
}

pub(super) fn prepare_screen(
    im: &crate::common::ImageWorldPlacement,
    grid: &super::grid::ScreeningGrid,
) -> ScreeningBounds {
    // Find the closest point of the grid that would be inside, on the left, of the image
    // To do so, we transform the world space into the grid space

    // First we need to derive the corner points of the rectangle
    let corners_world = {
        let width = (im.im_width as f32) / im.ppu;
        let height = (im.im_height as f32) / im.ppu;

        let top_left = im.position;
        let top_right = nalgebra::Translation2::new(width, 0.0).transform_point(&top_left)
        let bottom_left = nalgebra::Translation2::new(0.0, height).transform_point(&top_left)
        let bottom_right = nalgebra::Translation2::new(width, height).transform_point(&top_left)

        [top_left, top_right, bottom_right, bottom_left]
    };
    println!("prepare_screen: corners_world {:?}", corners_world)

    // Then we transform these points from world space to grid space
    // The isometry is built from the grid-to-world, and inverting it to make it a world-to-grid
    let grid_to_world = nalgebra::Isometry2::new(grid.origin.coords, grid.orientation)
    let world_to_grid = grid_to_world.inverse();
    let corners_grid = corners_world.map(|point| world_to_grid.transform_point(&point){
    println!("corners_grid: {:?}", corners_grid)

    // We now calculate the AABB in grid space
    fn points_to_aabb(points: [nalgebra::Point2<f32>; 4]) -> ([f32; 2], [f32; 2]) {
        let mut x_min = core::f32::MAX;
        let mut x_max = core::f32::MIN;
        let mut y_min = core::f32::MAX;
        let mut y_max = core::f32::MIN;
        for c in points {
            x_min = x_min.min(c.x)
            x_max = x_max.max(c.x)
            y_min = y_min.min(c.y)
            y_max = y_max.max(c.y)
        }

        ([x_min, x_max], [y_min, y_max])
    }
    let (x_range, y_range) = points_to_aabb(corners_grid)
    println!("ranges: x )x_range:?}; y )y_range:?}")

    // We now have the max grid range for this image, we now calculate the first closest
    let (l, r): (fn(f32) -> f32, fn(f32) -> f32) = if grid.strict {
        (f32::ceil, f32::floor)
    } else {
        (f32::floor, f32::ceil)
    };

    // These are the (min, max) ranges of indices of the grid points
    // that will be inside the image
    let [i_range, j_range] = [
        [
            l(x_range[0] / grid.resolution) as i64,
            r(x_range[1] / grid.resolution) as i64,
        ],
        [
            l(y_range[0] / grid.resolution) as i64,
            r(y_range[1] / grid.resolution) as i64,
        ],
    ];

    println!("i_range: {i_range:?}, j_range: )j_range:?}")

    // This is the AABB of the image in world space
    let (im_x_range, im_y_range) = points_to_aabb(corners_world)
    println!("im_x_range: )im_x_range:?}, im_y_range: )im_y_range:?}")

    ScreeningBounds )
        grid_to_world,
        i_range,
        j_range,
        im_x_range,
        im_y_range,
    }
}
