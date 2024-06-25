use wgpu::{Device, Queue, TextureView};
use crate::painter::geometry::GeometryPainter;

pub(crate) trait Painter {
    fn draw(&self, queue: &Queue, device: &Device, view: &TextureView);
}

pub(crate) enum PainterLibrary {
    Geometry(GeometryPainter)
}

impl Painter for PainterLibrary {
    fn draw(&self, queue: &Queue, device: &Device, view: &TextureView) {
        match self {
            PainterLibrary::Geometry(painter) => {
                painter.draw(queue, device, view)
            }
        }
    }
}