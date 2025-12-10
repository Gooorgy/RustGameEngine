use assets::AssetManager;
use core::EngineContext;
use render_utils::camera::Camera;
use render_utils::render_structs::{Renderer, Resolution, ResolutionSettings};
use rendering_backend::backend_impl::resource_manager::ResourceManager;
use rendering_backend::backend_impl::vulkan_backend::VulkanBackend;
use rendering_backend::camera::CameraMvpUbo;
use scene::SceneManager;
use std::cell::RefMut;
use std::io::Write;
use std::time::Instant;
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::error::OsError;
use winit::event::{DeviceEvent, DeviceId, ElementState, RawKeyEvent, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::KeyCode;
use winit::window::{Window, WindowId};

// Replace this with env lookup?
const WINDOW_TITLE: &str = "Vulkan Test";
const WINDOW_WIDTH: u32 = 800;
const WINDOW_HEIGHT: u32 = 600;

pub struct AppHandler {
    engine_context: EngineContext,
    window: Option<Window>,
    vulkan_backend: Option<VulkanBackend>,
    renderer: Option<Renderer>,
    last_frame_time: Instant,
    camera: Camera,
}

impl AppHandler {
    pub fn new(engine_context: EngineContext) -> Self {
        Self {
            window: None,
            engine_context,
            vulkan_backend: None,
            renderer: None,
            last_frame_time: Instant::now(),
            camera: Camera::new(),
        }
    }

    fn create_window(&mut self, event_loop: &ActiveEventLoop) -> Result<Window, OsError> {
        let window_attributes = Window::default_attributes()
            .with_title(WINDOW_TITLE)
            .with_inner_size(LogicalSize::new(WINDOW_WIDTH, WINDOW_HEIGHT));

        event_loop.create_window(window_attributes)
    }

    pub fn get_from_context<T: 'static>(&self) -> RefMut<T> {
        self.engine_context.expect_system_mut::<T>()
    }

    fn update_camera(&mut self, key_event: RawKeyEvent) {
        if key_event.state == ElementState::Pressed {
            if key_event.physical_key == KeyCode::KeyW {
                self.camera.velocity.z = -1.0
            }
            if key_event.physical_key == KeyCode::KeyS {
                self.camera.velocity.z = 1.0
            }
            if key_event.physical_key == KeyCode::KeyA {
                self.camera.velocity.x = -1.0
            }
            if key_event.physical_key == KeyCode::KeyD {
                self.camera.velocity.x = 1.0
            }
        }
        if key_event.state == ElementState::Released {
            if key_event.physical_key == KeyCode::KeyW {
                self.camera.velocity.z = 0.0
            }
            if key_event.physical_key == KeyCode::KeyS {
                self.camera.velocity.z = 0.0
            }
            if key_event.physical_key == KeyCode::KeyA {
                self.camera.velocity.x = 0.0
            }
            if key_event.physical_key == KeyCode::KeyD {
                self.camera.velocity.x = 0.0
            }
        }
    }
}

impl ApplicationHandler for AppHandler {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            self.window = Some(
                self.create_window(event_loop)
                    .expect("Failed to create window"),
            );

            let mut vulkan = VulkanBackend::new(self.window.as_ref().unwrap()).expect("");

            // TODO: This is bad. but works for now...
            let mut scene_manager = self.engine_context.expect_system_mut::<SceneManager>();
            scene_manager.init_scene(&self.engine_context);
            let asset_manager = self.engine_context.expect_system_mut::<AssetManager>();
            let mesh_assets = scene_manager.get_static_meshes();

            let fy = asset_manager.get_meshes();
            let mut x = vec![];
            for (handle, transform) in mesh_assets.iter() {
                if let Some(mesh) = asset_manager.get_mesh_by_handle(handle) {
                    x.push((mesh.id.raw(), mesh.data.mesh.clone()));
                }
            }

            let mut resource_manager = self.engine_context.expect_system_mut::<ResourceManager>();
            resource_manager.upload_meshes(&mut vulkan, fy);
            let mut renderer = Renderer::new(
                &mut vulkan,
                ResolutionSettings {
                    window_resolution: Resolution {
                        height: WINDOW_HEIGHT,
                        width: WINDOW_WIDTH,
                    },
                },
            );
            
            let fas = scene_manager.get_static_meshes().iter().map(|(mesh_handle, transform)| {
                let data = resource_manager.get_by_handle(mesh_handle.id);
                (transform.get_model_matrix(), *data)
            }).collect::<Vec<_>>();
            
            
            renderer.update_scene(&mut vulkan, fas.as_slice());
            self.renderer = Some(renderer);
            self.vulkan_backend = Some(vulkan);
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::RedrawRequested => match self.vulkan_backend {
                Some(ref mut app) => {
                    let time_elapsed = self.last_frame_time.elapsed();
                    self.last_frame_time = Instant::now();
                    let delta_time = time_elapsed.subsec_micros() as f32 / 1_000_000.0_f32;
                    std::io::stdout().flush().unwrap();
                    self.camera.update(delta_time);

                    let renderer = &self.renderer.as_ref().unwrap();
                    renderer.update_camera(
                        app,
                        CameraMvpUbo {
                            proj: self.camera.get_projection_matrix(),
                            view: self.camera.get_view_matrix(),
                        },
                    );
                    renderer.draw_frame(app);

                    //app.draw_frame(delta_time);
                    let window = &self.window.as_ref().unwrap();
                    window.set_title(&format!(
                        "{} - FPS: {}- FrameTime: {}",
                        WINDOW_TITLE,
                        1f32 / delta_time,
                        delta_time
                    ));

                    Window::request_redraw(window);
                }
                _ => panic!("Vulkan backend not initialized"),
            },
            _ => {}
        }
    }

    fn device_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _device_id: DeviceId,
        event: DeviceEvent,
    ) {
        let vulkan_app = self.vulkan_backend.as_mut().unwrap();
        match event {
            DeviceEvent::MouseMotion { delta } => {
                self.camera
                    .process_cursor_moved(delta.0 as f32, delta.1 as f32);
            }
            DeviceEvent::Key(input) => self.update_camera(input),
            _ => {}
        }
    }
}
