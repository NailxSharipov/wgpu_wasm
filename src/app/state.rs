use std::borrow::Cow;
use std::sync::{Arc, Mutex};
use log::info;
use wgpu::{Adapter, Device, Instance, Queue, RenderPipeline, Surface, SurfaceConfiguration};
use winit::application::ApplicationHandler;
use winit::event::{DeviceEvent, DeviceId, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::window::{Window, WindowId};

pub struct AppState {
    context: Arc<Mutex<Context>>,
}

impl AppState {
    pub fn new() -> Self {
        AppState { context: Arc::new(Mutex::from(Context { state: ContextState::None, graphic: None, counter: 0 })) }
    }
}

impl AppState {
    fn context_state(&self) -> ContextState {
        if let Ok(context) = self.context.lock() {
            context.state
        } else {
            ContextState::None
        }
    }

    fn update_state(&mut self, state: ContextState) {
        if let Ok(mut context) = self.context.lock() {
            context.state = state
        }
    }

    fn build_window(event_loop: &ActiveEventLoop) -> Window {
        #[cfg(target_arch = "wasm32")] {
            use winit::platform::web::WindowAttributesExtWebSys;
            use wasm_bindgen::JsCast;
            use web_sys::HtmlCanvasElement;

            let canvas = web_sys::window()
                .unwrap()
                .document()
                .unwrap()
                .get_element_by_id("canvas")
                .unwrap()
                .dyn_into::<HtmlCanvasElement>()
                .unwrap();
            let atts = Window::default_attributes().with_canvas(Some(canvas));
            event_loop.create_window(atts).unwrap()
        }

        #[cfg(not(target_arch = "wasm32"))] {
            event_loop.create_window(Window::default_attributes()).unwrap()
        }
    }

    async fn init_graphic(window: Window) -> GraphicContext {
        let instance = Instance::default();
        let win = Arc::new(window);
        let surface = instance.create_surface(win.clone()).unwrap();

        let (adapter, device, queue) = Self::init_device(instance, &surface).await;

        // Load the shaders from disk
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("shader.wgsl"))),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[],
            push_constant_ranges: &[],
        });

        let swapchain_capabilities = surface.get_capabilities(&adapter);
        let swapchain_format = swapchain_capabilities.formats[0];

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
                targets: &[Some(swapchain_format.into())],
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
        });

        let mut size = win.inner_size();
        size.width = size.width.max(1);
        size.height = size.height.max(1);

        let surface_config = surface
            .get_default_config(&adapter, size.width, size.height)
            .unwrap();

        surface.configure(&device, &surface_config);

        let render = RenderContext {
            device,
            queue,
            render_pipeline,
        };

        GraphicContext {
            window: win,
            surface,
            surface_config,
            render,
        }
    }

    async fn init_device(instance: Instance, surface: &Surface<'_>) -> (Adapter, Device, Queue) {
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                force_fallback_adapter: false,
                // Request an adapter which can render to our surface
                compatible_surface: Some(surface),
            })
            .await
            .expect("Failed to find an appropriate adapter");

        // Create the logical device and command queue
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    required_features: wgpu::Features::empty(),
                    // Make sure we use the texture resolution limits from the adapter, so we can support images the size of the swapchain.
                    required_limits: wgpu::Limits::downlevel_webgl2_defaults()
                        .using_resolution(adapter.limits()),
                },
                None,
            )
            .await
            .expect("Failed to create device");

        (adapter, device, queue)
    }

    fn init_window(&mut self, event_loop: &ActiveEventLoop) {
        self.update_state(ContextState::Initializing);

        let window = Self::build_window(event_loop);

        let clone_context = Arc::clone(&self.context);
        #[cfg(not(target_arch = "wasm32"))] {
            use pollster::FutureExt;
            let graphic = Self::init_graphic(window).block_on();
            if let Ok(mut context) = clone_context.lock() {
                context.graphic = Some(graphic);
                context.state = ContextState::Ready;
            };
        }

        #[cfg(target_arch = "wasm32")] {
            use wasm_bindgen_futures::spawn_local;
            spawn_local(async move {
                let graphic = Self::init_graphic(window).await;
                if let Ok(mut context) = clone_context.lock() {
                    context.graphic = Some(graphic);
                    context.state = ContextState::Ready;
                };
            });
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ContextState {
    None,
    Initializing,
    Ready,
}

pub struct Context {
    state: ContextState,
    graphic: Option<GraphicContext>,
    counter: i32,
}

impl Context {}

struct GraphicContext {
    window: Arc<Window>,
    surface: Surface<'static>,
    surface_config: SurfaceConfiguration,
    render: RenderContext,
}

struct RenderContext {
    device: Device,
    queue: Queue,
    render_pipeline: RenderPipeline,
}

impl GraphicContext {
    fn handle_window_event(&mut self, event: WindowEvent) -> bool {
        match event {
            WindowEvent::Resized(new_size) => {
                self.surface_config.width = new_size.width.clamp(100, 1600);
                self.surface_config.height = new_size.height.clamp(100, 1600);

                info!("w: {}, h: {}", new_size.width, new_size.height);

                self.surface.configure(&self.render.device, &self.surface_config);
                // On macos the window needs to be redrawn manually after resizing
                // self.window.request_redraw();
            }
            WindowEvent::RedrawRequested => {
                let frame = self.surface
                    .get_current_texture()
                    .expect("Failed to acquire next swap chain texture");
                let view = frame
                    .texture
                    .create_view(&wgpu::TextureViewDescriptor::default());
                let mut encoder =
                    self.render.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
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
                                    load: wgpu::LoadOp::Clear(wgpu::Color::GREEN),
                                    store: wgpu::StoreOp::Store,
                                },
                            })],
                            depth_stencil_attachment: None,
                            timestamp_writes: None,
                            occlusion_query_set: None,
                        });
                    rpass.set_pipeline(&self.render.render_pipeline);
                    rpass.draw(0..3, 0..1);
                }

                self.render.queue.submit(Some(encoder.finish()));
                frame.present();
            }
            WindowEvent::CloseRequested => return true,
            _ => {}
        };

        false
    }
}

impl ApplicationHandler for AppState {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.context_state() == ContextState::None {
            info!("start init_window");
            self.init_window(event_loop);
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _window_id: WindowId, event: WindowEvent) {
        if self.context_state() != ContextState::Ready {
            return;
        }
        if let Ok(context) = &mut self.context.lock() {
            if let Some(graphic) = &mut context.graphic {
                if graphic.handle_window_event(event) {
                    event_loop.exit();
                }
            }
        }
    }

    fn device_event(&mut self, _event_loop: &ActiveEventLoop, _device_id: DeviceId, _event: DeviceEvent) {}

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        if self.context_state() != ContextState::Ready {
            return;
        }

        if let Ok(context) = &mut self.context.lock() {
            if let Some(graphic) = &mut context.graphic {
                graphic.window.request_redraw();
                context.counter += 1;
            }
        }
    }
}
