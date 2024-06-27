use std::borrow::Cow;
use wgpu::{Buffer, BufferUsages, ColorTargetState, Device, Queue, RenderPipeline, TextureView, util::DeviceExt, BindGroup, BufferBindingType, ShaderStages};
use crate::draw::brush::Brush;
use crate::draw::document::Document;
use crate::draw::painter::Painter;

pub(crate) struct GeometryPainter {
    document: Document,
    render_pipeline: RenderPipeline,
    vertex_buffer: Buffer,
    brush_buffer: Buffer,
    index_buffer: Buffer,
    transform_buffer: Buffer,
    bind_group: BindGroup,
}

impl GeometryPainter {
    pub(crate) fn create(color: ColorTargetState, device: &Device, screen_width: u32, screen_height: u32) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("shader.wgsl"))),
        });

        let document = Document::random(1000, 1000, 1.0, 100_000);
        let mesh = &document.mesh;

        // Create GPU buffers
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(&mesh.points.iter().map(|p| [p.x, p.y]).collect::<Vec<_>>()),
            usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
        });

        let brush_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Brush Buffer"),
            contents: bytemuck::cast_slice(&mesh.brushes),
            usage: BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(&mesh.indices),
            usage: BufferUsages::INDEX,
        });

        let ortho_matrix = create_orthographic_matrix(screen_width as f32, screen_height as f32, document.width as f32, document.height as f32);
        let transform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Transform Buffer"),
            contents: bytemuck::cast_slice(&ortho_matrix),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });

        let brush_data: Vec<[f32; 8]> = Brush::create_set().iter().map(|brush| {
            [
                brush.vec.x,
                brush.vec.y,
                brush.width,
                brush.color.r,
                brush.color.g,
                brush.color.b,
                0.0,
                0.0,
            ]
        }).collect();

        let brush_set_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Brush Data Buffer"),
            contents: bytemuck::cast_slice(&brush_data),
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("Bind Group Layout"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: wgpu::BufferSize::new(512), // Updated size
                    },
                    count: None,
                },
            ],
        });

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Bind Group"),
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: transform_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: brush_set_buffer.as_entire_binding(),
                },
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let color_target_state = ColorTargetState {
            format: color.format,
            blend: Some(wgpu::BlendState::ALPHA_BLENDING),
            write_mask: wgpu::ColorWrites::ALL,
        };

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                compilation_options: Default::default(),
                buffers: &[
                    wgpu::VertexBufferLayout {
                        array_stride: std::mem::size_of::<[f32; 2]>() as wgpu::BufferAddress,
                        step_mode: wgpu::VertexStepMode::Vertex,
                        attributes: &[
                            wgpu::VertexAttribute {
                                offset: 0,
                                shader_location: 0,
                                format: wgpu::VertexFormat::Float32x2,
                            },
                        ],
                    },
                    wgpu::VertexBufferLayout {
                        array_stride: std::mem::size_of::<u32>() as wgpu::BufferAddress,
                        step_mode: wgpu::VertexStepMode::Vertex,
                        attributes: &[
                            wgpu::VertexAttribute {
                                offset: 0,
                                shader_location: 1,
                                format: wgpu::VertexFormat::Uint32,
                            },
                        ],
                    },
                ],
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                compilation_options: Default::default(),
                targets: &[Some(color_target_state)],
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        Self { document, render_pipeline, vertex_buffer, brush_buffer, index_buffer, transform_buffer, bind_group }
    }

    fn update_vertex_buffer(&self, device: &Device, queue: &Queue) {
        let mesh = &self.document.mesh;

        // Create a staging buffer with the updated vertex data
        let staging_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Staging Buffer"),
            contents: bytemuck::cast_slice(&mesh.points.iter().map(|p| [p.x, p.y]).collect::<Vec<_>>()),
            usage: BufferUsages::MAP_WRITE | BufferUsages::COPY_SRC,
        });

        // Create a command encoder
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Update Vertex Buffer Encoder"),
        });

        // Copy data from the staging buffer to the vertex buffer
        encoder.copy_buffer_to_buffer(
            &staging_buffer,
            0,
            &self.vertex_buffer,
            0,
            (mesh.points.len() * std::mem::size_of::<[f32; 2]>()) as wgpu::BufferAddress,
        );

        // Submit the command buffer
        queue.submit(Some(encoder.finish()));
    }
}

impl Painter for GeometryPainter {
    fn draw(&mut self, queue: &Queue, device: &Device, view: &TextureView) {
        self.document.update();
        self.update_vertex_buffer(device, queue);

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: None,
        });
        {
            let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
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
            rpass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
            rpass.set_vertex_buffer(1, self.brush_buffer.slice(..));
            rpass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
            rpass.set_bind_group(0, &self.bind_group, &[]);
            rpass.draw_indexed(0..self.document.mesh.indices.len() as u32, 0, 0..1);
        }

        queue.submit(Some(encoder.finish()));
    }
}

// Function to create an orthographic projection matrix
fn create_orthographic_matrix(screen_width: f32, screen_height: f32, doc_width: f32, doc_height: f32) -> [f32; 16] {
    let aspect_ratio = doc_width / doc_height;
    let scaled_width = screen_height * aspect_ratio;

    let right = scaled_width;
    let left = 0.0;
    let top = 0.0;
    let bottom = screen_height;

    [
        2.0 / (right - left), 0.0, 0.0, 0.0,
        0.0, 2.0 / (top - bottom), 0.0, 0.0,
        0.0, 0.0, -1.0, 0.0,
        -(right + left) / (right - left), -(top + bottom) / (top - bottom), 0.0, 1.0,
    ]
}
