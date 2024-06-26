use wgpu::Color;
use crate::draw::point::Point;

pub(crate) struct Mesh {
    pub(crate) points: Vec<Point>,
    pub(crate) colors: Vec<Color>,
    pub(crate) indices: Vec<u32>, // we will use clockwise order for a face side
}

impl Mesh {

    pub(crate) fn with_capacity(capacity: usize) -> Self {
        Self { points: Vec::with_capacity(capacity), colors: Vec::with_capacity(capacity), indices: Vec::with_capacity(3 * capacity) }
    }

    pub(crate) fn append(&mut self, mut mesh: Mesh) {
        let offset = self.points.len() as u32;

        self.points.append(&mut mesh.points);
        self.colors.append(&mut mesh.colors);

        let mut indices = mesh.indices;
        for i in indices.iter_mut() {
            *i += offset;
        }

        self.indices.append(&mut indices);
    }
}