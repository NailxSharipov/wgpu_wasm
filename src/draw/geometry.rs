use std::borrow::Cow;
use wgpu::{Buffer, BufferUsages, ColorTargetState, Device, Queue, RenderPipeline, TextureView, util::DeviceExt, BindGroup, BufferBindingType, ShaderStages, BufferAddress, BindGroupLayout};
use crate::draw::brush::Brush;
use crate::draw::document::Document;
use crate::draw::painter::Painter;
use crate::draw::point::Point;

struct Screen {
    width: f32,
    height: f32,
    pos: Point,
    scale: f32,
    is_modified: bool,
}

pub(crate) struct GeometryPainter {
    pub(crate) document: Document,
    screen: Screen,
    render_pipeline: RenderPipeline,
    vertex_buffer: Buffer,
    brush_buffer: Buffer,
    index_buffer: Buffer,
    transform_buffer: Buffer,
    bind_group: BindGroup,
}

impl GeometryPainter {
    pub(crate) fn create(color: ColorTargetState, device: &Device, screen_width: u32, screen_height: u32) -> Self {
        let document = Document::random(1000, 1000, 1.0, 1_00);

        let width = screen_width as f32;
        let height = screen_height as f32;

        let screen = Screen {
            width,
            height,
            pos: Point { x: 0.5 * width, y: 0.5 * height },
            scale: 1.0,
            is_modified: true,
        };

        // Create GPU buffers
        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(&document.mesh.points.iter().map(|p| [p.x, p.y]).collect::<Vec<_>>()),
            usage: BufferUsages::VERTEX | BufferUsages::COPY_DST,
        });

        let brush_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Brush Buffer"),
            contents: bytemuck::cast_slice(&document.mesh.brushes),
            usage: BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(&document.mesh.indices),
            usage: BufferUsages::INDEX,
        });

        let ortho_matrix = create_orthographic_matrix_with_camera(
            &screen,
            document.width,
            document.height,
        );
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

        let render_pipeline = Self::build_pipeline(color, device, bind_group_layout);

        Self { document, screen, render_pipeline, vertex_buffer, brush_buffer, index_buffer, transform_buffer, bind_group }
    }


    fn build_pipeline(color: ColorTargetState, device: &Device, bind_group_layout: BindGroupLayout) -> RenderPipeline {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("shader.wgsl"))),
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

        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                compilation_options: Default::default(),
                buffers: &[
                    wgpu::VertexBufferLayout {
                        array_stride: std::mem::size_of::<[f32; 2]>() as BufferAddress,
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
                        array_stride: std::mem::size_of::<u32>() as BufferAddress,
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
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Cw,
                cull_mode: Some(wgpu::Face::Front),
                unclipped_depth: false,
                polygon_mode: wgpu::PolygonMode::Fill,
                conservative: false,
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            multiview: None,
        })
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

    fn update_transform_buffer(&mut self, device: &Device, queue: &Queue) {
        if !self.screen.is_modified {
            return;
        }
        self.screen.is_modified = false;

        let ortho_matrix = create_orthographic_matrix_with_camera(
            &self.screen,
            self.document.width,
            self.document.height,
        );

        let staging_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Staging Buffer"),
            contents: bytemuck::cast_slice(&ortho_matrix),
            usage: BufferUsages::MAP_WRITE | BufferUsages::COPY_SRC,
        });

        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Update Transform Buffer Encoder"),
        });

        encoder.copy_buffer_to_buffer(
            &staging_buffer,
            0,
            &self.transform_buffer,
            0,
            std::mem::size_of::<[f32; 16]>() as BufferAddress,
        );

        queue.submit(Some(encoder.finish()));
    }
}

impl Painter for GeometryPainter {
    fn draw(&mut self, queue: &Queue, device: &Device, view: &TextureView) {
        self.document.update();
        self.update_vertex_buffer(device, queue);
        self.update_transform_buffer(device, queue);

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

    fn update_size(&mut self, screen_width: u32, screen_height: u32) {
        self.screen.width = screen_width as f32;
        self.screen.height = screen_height as f32;
        self.screen.is_modified = true;
    }

    fn update_scale(&mut self, scale: f32) {
        self.screen.scale = scale;
        self.screen.is_modified = true;
    }

    fn update_pos(&mut self, pos: Point) {
        self.screen.pos = pos;
        self.screen.is_modified = true;
    }
}

fn create_orthographic_matrix_with_camera(
    screen: &Screen,
    doc_width: f32,
    doc_height: f32,
) -> [f32; 16] {
    let aspect_ratio = doc_width / doc_height;
    let scaled_width = screen.height * aspect_ratio / screen.scale;
    let scaled_height = screen.height / screen.scale;

    let right = screen.pos.x + scaled_width * 0.5;
    let left = screen.pos.x - scaled_width * 0.5;
    let top = screen.pos.y - scaled_height * 0.5;
    let bottom = screen.pos.y + scaled_height * 0.5;

    [
        2.0 / (right - left), 0.0, 0.0, 0.0,
        0.0, 2.0 / (top - bottom), 0.0, 0.0,
        0.0, 0.0, -1.0, 0.0,
        -(right + left) / (right - left), -(top + bottom) / (top - bottom), 0.0, 1.0,
    ]

    // let aspect_ratio = doc_width / doc_height;
    // let scaled_width = screen.height * aspect_ratio;
    //
    // let right = scaled_width;
    // let left = 0.0;
    // let top = 0.0;
    // let bottom = screen.height;
    //
    // [
    //     2.0 / (right - left), 0.0, 0.0, 0.0,
    //     0.0, 2.0 / (top - bottom), 0.0, 0.0,
    //     0.0, 0.0, -1.0, 0.0,
    //     -(right + left) / (right - left), -(top + bottom) / (top - bottom), 0.0, 1.0,
    // ]
}
