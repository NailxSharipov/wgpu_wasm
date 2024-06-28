use rand::Rng;
use tracing::info;
use crate::draw::brush::MAX_BRUSHES;
use crate::draw::mesh::Mesh;
use crate::draw::point::Point;
use crate::draw::rect::Rect;

pub(crate) struct Document {
    pub(crate) width: f32,
    pub(crate) height: f32,
    pub(crate) stroke: f32,
    pub(crate) rects: Vec<Rect>,
    pub(crate) positions: Vec<Point>,
    pub(crate) velocities: Vec<Point>,
    pub(crate) mesh: Mesh,
    pub(crate) active: usize,
    pub(crate) new_count: usize,
    pub(crate) count: usize
}

impl Document {
    pub(crate) fn random(width: u32, height: u32, stroke: f32, count: usize) -> Self {
        let mut rng = rand::thread_rng();

        let mut rects = Vec::with_capacity(count);
        let mut positions = Vec::with_capacity(count);
        let mut velocities = Vec::with_capacity(count);

        let a = 32;

        let max_width = width / a;
        let max_height = height / a;
        let min_width = max_width / 4;
        let min_height = max_height / 4;

        let x_range = 0..(width - max_width);
        let y_range = 0..(height - max_height);
        let w_range = min_width..max_width;
        let h_range = min_height..max_height;

        for i in 0..count {
            let x = rng.gen_range(x_range.clone());
            let y = rng.gen_range(y_range.clone());

            let vx = 0.1f32 * (rng.gen::<f32>() - 0.5f32);
            let vy = 0.1f32 * (rng.gen::<f32>() - 0.5f32);

            positions.push(Point { x: x as f32, y: y as f32 });
            velocities.push(Point { x: vx, y: vy });

            let w = rng.gen_range(w_range.clone());
            let h = rng.gen_range(h_range.clone());

            let brush = (i % MAX_BRUSHES) as u32;

            let rect = Rect {
                width: w,
                height: h,
                brush,
            };

            rects.push(rect);
        }

        let mut mesh = Mesh::with_capacity(4 * rects.len());

        for (i, rect) in rects.iter().enumerate() {
            mesh.append(rect.mesh(positions[i], stroke));
        }

        Self {
            width: width as f32,
            height: height as f32,
            stroke,
            rects,
            positions,
            velocities,
            mesh,
            active: 0,
            new_count: count,
            count,
        }
    }

    pub(crate) fn update(&mut self) {
        let dt = 50.0;
        let s = self.stroke;
        let a = 1;
        let mut i = self.active % a;
        while i < self.rects.len() {
            let rect = &self.rects[i];
            let mut pos = self.positions[i];
            let mut vel = self.velocities[i];
            if pos.x > self.width && vel.x > 0.0 || pos.x < 0.0 && vel.x < 0.0 {
                vel.x = -vel.x;
            }
            if pos.y > self.height && vel.y > 0.0 || pos.y < 0.0 && vel.y < 0.0 {
                vel.y = -vel.y;
            }

            pos.x += vel.x * dt;
            pos.y += vel.y * dt;
            self.positions[i] = pos;
            self.velocities[i] = vel;

            rect.update(pos, s, i, &mut self.mesh);

            i += a;
        }
    }

    pub (crate) fn update_count(&mut self, count: usize) {
        self.new_count = count;
        info!("count {}", count);
    }
}