use wgpu::{Device, Queue, TextureView};
use crate::draw::geometry::GeometryPainter;

pub(crate) trait Painter {
    fn draw(&mut self, queue: &Queue, device: &Device, view: &TextureView);
}

pub(crate) enum PainterLibrary {
    Geometry(GeometryPainter)
}

impl Painter for PainterLibrary {
    fn draw(&mut self, queue: &Queue, device: &Device, view: &TextureView) {
        match self {
            PainterLibrary::Geometry(painter) => {
                painter.draw(queue, device, view)
            }
        }
    }
}