use crate::app_handler::AppHandler;
use core::EngineContext;
use winit::event_loop::EventLoop;

/// Entry point for the engine. Creates and owns the winit event loop and the
/// application handler. Construct with `App::new`, set up your scene through
/// `engine_context_mut`, then call `run` to enter the event loop.
pub struct App {
    event_loop: EventLoop<()>,
    app_handler: AppHandler,
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

impl App {
    /// Creates the engine with a fresh `EngineContext` and a winit event loop.
    pub fn new() -> App {
        let engine_context = EngineContext::new();
        let event_loop = EventLoop::new().expect("Failed to create event loop");
        let app_handler = AppHandler::new(engine_context);
        Self {
            event_loop,
            app_handler,
        }
    }

    /// Returns a mutable reference to the `EngineContext` for scene setup.
    pub fn engine_context_mut(&mut self) -> &mut EngineContext {
        self.app_handler.engine_context_mut()
    }

    /// Starts the winit event loop. Blocks until the window is closed.
    pub fn run(mut self) {
        self.event_loop
            .run_app(&mut self.app_handler)
            .expect("Failed to run event loop");
    }
}
