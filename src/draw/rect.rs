use crate::draw::brush::MAX_BRUSHES;
use crate::draw::mesh::Mesh;
use crate::draw::point::Point;

pub(crate) struct Rect {
    pub(crate) width: u32,
    pub(crate) height: u32,
    pub(crate) brush: u32, // index to brush
}

impl Rect {
    pub(crate) fn mesh(&self, p: Point, s: f32) -> Mesh {
        let points = self.points(p, s).to_vec();

        let indices = vec![
            0, 1, 3, 1, 2, 3,
            8, 4, 9, 9, 4, 5,
            5, 10, 9, 10, 5, 6,
            10, 6, 11, 11, 6, 7,
            7, 4, 8, 8, 11, 7,
        ];
        let mut brushes = vec![self.brush; 12];
        for i in 4..12 {
            brushes[i] += MAX_BRUSHES as u32;
        }

        Mesh { points, brushes, indices }
    }

    pub(crate) fn update(&self, p: Point, s: f32, index: usize, mesh: &mut Mesh) {
        let points = self.points(p, s);
        let j = index * 12;
        for i in 0..12 {
            mesh.points[i + j] = points[i];
        }
    }

    #[inline(always)]
    fn points(&self, p: Point, s: f32) -> [Point; 12] {
        let p0 = Point { x: p.x, y: p.y };
        let p1 = Point { x: p.x, y: p.y + self.height as f32 };
        let p2 = Point { x: p.x + self.width as f32, y: p.y + self.height as f32 };
        let p3 = Point { x: p.x + self.width as f32, y: p.y };

        let q0 = Point { x: p0.x - s, y: p0.y - s };
        let q1 = Point { x: p1.x - s, y: p1.y + s };
        let q2 = Point { x: p2.x + s, y: p2.y + s };
        let q3 = Point { x: p3.x + s, y: p3.y - s };

        let g0 = Point { x: p0.x + s, y: p0.y + s };
        let g1 = Point { x: p1.x + s, y: p1.y - s };
        let g2 = Point { x: p2.x - s, y: p2.y - s };
        let g3 = Point { x: p3.x - s, y: p3.y + s };

        [p0, p1, p2, p3, q0, q1, q2, q3, g0, g1, g2, g3]
    }
}