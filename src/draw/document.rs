use rand::Rng;
use wgpu::Color;
use crate::draw::mesh::Mesh;
use crate::draw::rect::Rect;

pub(crate) struct Document {
    pub(crate) width: u32,
    pub(crate) height: u32,
    pub(crate) rects: Vec<Rect>,
}

impl Document {
    pub(crate) fn random(width: u32, height: u32, count: usize) -> Self {
        let mut rng = rand::thread_rng();
        let mut rects = Vec::with_capacity(count);

        let max_width = width / 16;
        let max_height = height / 16;

        let x_range = 0..(width - max_width);
        let y_range = 0..(height - max_height);
        let w_range = 0..max_width;
        let h_range = 0..max_height;

        for _ in 0..count {
            let x = rng.gen_range(&x_range);
            let y = rng.gen_range(&y_range);
            let w = rng.gen_range(&w_range);
            let h = rng.gen_range(&h_range);

            let color = Color {
                r: rng.gen(),
                g: rng.gen(),
                b: rng.gen(),
                a: 1.0,
            };

            let rect = Rect {
                x,
                y,
                width: w,
                height: h,
                color,
            };

            rects.push(rect);
        }

        Self { width, height, rects }
    }

    pub(crate) fn mesh(&self) -> Mesh {
        let mut mesh = Mesh::with_capacity(4 * self.rects.len());

        for rect in self.rects.iter() {
            mesh.append(rect.mesh());
        }

        mesh
    }
}