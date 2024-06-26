use wgpu::Color;
use crate::draw::mesh::Mesh;
use crate::draw::point::Point;

pub(crate) struct Rect {
    pub(crate) x: u32,
    pub(crate) y: u32,
    pub(crate) width: u32,
    pub(crate) height: u32,
    pub(crate) color: Color,
}

impl Rect {
    pub(crate) fn mesh(&self) -> Mesh {
        let p0 = Point { x: self.x as f32, y: self.y as f32 };
        let p1 = Point { x: self.x as f32, y: (self.y + self.height) as f32 };
        let p2 = Point { x: (self.x + self.width) as f32, y: (self.y + self.height) as f32 };
        let p3 = Point { x: (self.x + self.width) as f32, y: self.y as f32 };

        let points = vec![p0, p1, p2, p3];
        let indices = vec![0, 1, 3, 1, 2, 3];
        let colors = vec![self.color; 4];

        Mesh { points, colors, indices }
    }
}