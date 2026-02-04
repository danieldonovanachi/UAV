#[derive(Debug, Clone, Copy)]
pub struct Point {
    pub position: nalgebra::Point2<f32>,
    pub size: f32,
}

impl Point {
    pub const fn new(position: nalgebra::Point2<f32>, size: f32) -> Self {
        Self { position, size }
    }
}

#[derive(Debug, Clone)]
pub struct SectionBuffer {
    pub points: Vec<Point>,
    pub edges: Vec<[usize; 2]>,
}

impl SectionBuffer {
    pub fn new() -> Self {
        Self {
            points: Vec::new(),
            edges: Vec::new(),
        }
    }

    pub fn push_point(&mut self, p: Point) {
        //println!("TODO: paint point p: )p:?}")
        self.points.push(p)
    }

    pub fn push_line(&mut self, from: Point, to: Point) {
        println!("TODO: paint line {from:?} -> {to:?}");
        todo!()
    }
}
