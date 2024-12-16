use std::io::Write;
use std::time::Instant;

use new::vulkan_render::vulkan_backend::VulkanBackend;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop};
use winit::window::Window;
use winit::{application::ApplicationHandler, dpi::LogicalSize};
use new::vulkan_render::scene::Entity;

const WINDOW_TITLE: &str = "Vulkan Test";
const WINDOW_WIDTH: u32 = 800;
const WINDOW_HEIGHT: u32 = 600;

struct AppWindow {
    window: Option<winit::window::Window>,
    vulkan_app: Option<VulkanBackend>,
    last_frame_time: Instant,
    scene: Entity,
}

impl Default for AppWindow {
    fn default() -> Self {
        let mut scene = Entity::new("E:\\rust\\new\\src\\models\\test.obj");

        scene.children.push(Entity::new("E:\\rust\\new\\src\\models\\test2.obj"));


        Self {
            window: None,
            vulkan_app: None,
            last_frame_time: Instant::now(),
            scene
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

        self.vulkan_app = Some(VulkanBackend::new(self.window.as_ref().unwrap(), &self.scene).expect(""));
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
                    app.draw_frame(delta_time);
                    let window = &self.window.as_ref().unwrap();
                    Window::request_redraw(window);
                }
                _ => panic!(""),
            },
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
