use core::EngineContext;
use scene::SceneManager;
use std::cell::RefMut;
use std::io::Write;
use std::time::Instant;
use vulkan_backend::scene::Transform;
use vulkan_backend::vulkan_backend::VulkanBackend;
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::error::OsError;
use winit::event::{DeviceEvent, DeviceId, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::window::{Window, WindowId};

// Replace this with env lookup?
const WINDOW_TITLE: &str = "Vulkan Test";
const WINDOW_WIDTH: u32 = 800;
const WINDOW_HEIGHT: u32 = 600;

pub struct AppHandler {
    engine_context: EngineContext,
    window: Option<Window>,
    vulkan_backend: Option<VulkanBackend>,
    last_frame_time: Instant,
}

impl AppHandler {
    pub fn new(engine_context: EngineContext) -> Self {
        Self {
            window: None,
            engine_context,
            vulkan_backend: None,
            last_frame_time: Instant::now(),
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

            let mesh_assets = scene_manager
                .get_static_meshes()
                .iter()
                .map(|asse| asse.data.mesh.clone())
                .collect::<Vec<_>>();
            vulkan.upload_meshes(mesh_assets, Transform::default().model);

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
                    app.camera.update(delta_time);
                    app.draw_frame(delta_time);
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
                vulkan_app
                    .camera
                    .process_cursor_moved(delta.0 as f32, delta.1 as f32);
            }
            DeviceEvent::Key(input) => vulkan_app.camera.process_keyboard_event(input),
            _ => {}
        }
    }
}
