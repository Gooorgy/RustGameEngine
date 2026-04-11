use crate::engine::Engine;
use core::EngineContext;
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::event::{DeviceEvent, DeviceId, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::window::{Window, WindowId};

const WINDOW_TITLE: &str = "Vulkan Test";
const WINDOW_WIDTH: u32 = 800;
const WINDOW_HEIGHT: u32 = 600;

/// Winit `ApplicationHandler` implementation. Thin OS/event-loop adapter.
/// Holds the pre-configured `EngineContext` until the window is ready, then
/// constructs an `Engine` and forwards all events to it.
pub struct AppHandler {
    context: Option<EngineContext>,
    engine: Option<Engine>,
}

impl AppHandler {
    pub fn new(context: EngineContext) -> Self {
        Self {
            context: Some(context),
            engine: None,
        }
    }

    fn create_window(&self, event_loop: &ActiveEventLoop) -> Window {
        let attrs = Window::default_attributes()
            .with_title(WINDOW_TITLE)
            .with_inner_size(LogicalSize::new(WINDOW_WIDTH, WINDOW_HEIGHT));
        event_loop.create_window(attrs).expect("Failed to create window")
    }
}

impl ApplicationHandler for AppHandler {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.engine.is_none() {
            let window = self.create_window(event_loop);
            let context = self.context.take().expect("EngineContext already consumed");
            self.engine = Some(Engine::new(window, context));
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
            WindowEvent::RedrawRequested => {
                if let Some(engine) = &mut self.engine {
                    engine.tick();
                }
            }
            _ => {}
        }
    }

    fn device_event(
        &mut self,
        _event_loop: &ActiveEventLoop,
        _device_id: DeviceId,
        event: DeviceEvent,
    ) {
        if let Some(engine) = &mut self.engine {
            engine.handle_device_event(event);
        }
    }
}
