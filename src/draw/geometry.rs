use std::borrow::Cow;
use wgpu::{ColorTargetState, Device, Queue, RenderPipeline, TextureView};
use crate::draw::document::Document;
use crate::draw::mesh::Mesh;
use crate::draw::painter::Painter;

pub(crate) struct GeometryPainter {
    pub(crate) document: Document,
    mesh: Mesh,
    render_pipeline: RenderPipeline,
}

impl GeometryPainter {
    pub(crate) fn create(color: ColorTargetState, device: &Device) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("shader.wgsl"))),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[],
            push_constant_ranges: &[],
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                buffers: &[],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                compilation_options: Default::default(),
                targets: &[Some(color)],
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        let document = Document::random(10000, 10000, 1000);
        let mesh = document.mesh();

        Self { document, mesh, render_pipeline }
    }
}


impl Painter for GeometryPainter {
    fn draw(&self, queue: &Queue, device: &Device, view: &TextureView) {
        let mut encoder =
            device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: None,
            });
        {
            let mut rpass =
                encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: None,
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view: &view,
                        resolve_target: None,
                        ops: wgpu::Operations {
                            load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                            store: wgpu::StoreOp::Store,
                        },
                    })],
                    depth_stencil_attachment: None,
                    timestamp_writes: None,
                    occlusion_query_set: None,
                });
            rpass.set_pipeline(&self.render_pipeline);
            rpass.draw(0..3, 0..1);
        }

        queue.submit(Some(encoder.finish()));
    }
}