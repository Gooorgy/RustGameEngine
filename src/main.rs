use std::cell::RefCell;
use std::io::Write;
use std::rc::Rc;
use std::time::Instant;
use typed_arena::Arena;
use new::vulkan_render::vulkan_backend::VulkanBackend;
use winit::event::{DeviceEvent, DeviceId, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::Window;
use winit::{application::ApplicationHandler, dpi::LogicalSize};
use new::vulkan_render::scene;
use new::vulkan_render::scene::{SceneNode};

const WINDOW_TITLE: &str = "Vulkan Test";
const WINDOW_WIDTH: u32 = 800;
const WINDOW_HEIGHT: u32 = 600;
struct AppWindow {
    window: Option<winit::window::Window>,
    vulkan_app: Option<VulkanBackend>,
    last_frame_time: Instant,
    scene: Rc<RefCell<SceneNode>>,
}

impl Default for AppWindow {
    fn default() -> Self {
        let mut scene_root = SceneNode::new("E:\\rust\\new\\src\\models\\test.obj");

        //scene_root.borrow_mut().add_child("E:\\rust\\new\\src\\models\\test2.obj");

        SceneNode::add_child(scene_root.clone(), "E:\\rust\\new\\src\\models\\test2.obj");
        SceneNode::update(scene_root.clone());
        Self {
            window: None,
            vulkan_app: None,
            last_frame_time: Instant::now(),
            scene: scene_root,
        }
    }
}

impl ApplicationHandler for AppWindow {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let monitor = event_loop.primary_monitor();
        let _x = Some(winit::window::Fullscreen::Borderless(monitor));
        let window_attributes = Window::default_attributes()
            .with_title(WINDOW_TITLE)
            .with_inner_size(LogicalSize::new(WINDOW_WIDTH, WINDOW_HEIGHT));
        // .with_fullscreen(x);
        self.window = Some(event_loop.create_window(window_attributes).unwrap());

        self.vulkan_app = Some(VulkanBackend::new(self.window.as_ref().unwrap(), self.scene.clone()).expect(""));
    }

    // Handle window event
    fn window_event(
        &mut self,
        event_loop: &winit::event_loop::ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: winit::event::WindowEvent,
    ) {
        match event {
            WindowEvent::CloseRequested => event_loop.exit(),
            WindowEvent::RedrawRequested => match self.vulkan_app {
                Some(ref mut app) => {
                    let time_elapsed = self.last_frame_time.elapsed();
                    self.last_frame_time = Instant::now();
                    let delta_time = time_elapsed.subsec_micros() as f32 / 1_000_000.0_f32;
                    print!("\r{}", delta_time);
                    std::io::stdout().flush().unwrap();
                    app.camera.update(delta_time);
                    app.draw_frame(delta_time);
                    let window = &self.window.as_ref().unwrap();
                    Window::request_redraw(window);
                }
                _ => panic!(""),
            },
            _ => {}
        }
    }

    fn device_event(&mut self, event_loop: &ActiveEventLoop, device_id: DeviceId, event: DeviceEvent) {
        let vulkan_app = self.vulkan_app.as_mut().unwrap();
        match event {
            DeviceEvent::MouseMotion { delta } => {
                vulkan_app.camera.process_cursor_moved(delta.0 as f32, delta.1 as f32);
            },
            DeviceEvent::Key(input) => {
                vulkan_app.camera.process_keyboard_event(input)
            }
            _ => {}
        }
    }
}

fn main() {
    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);
    let mut app_window = AppWindow::default();
    let _ = event_loop.run_app(&mut app_window);
}
