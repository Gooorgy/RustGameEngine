use crate::app_handler::AppHandler;
use core::EngineContext;
use winit::event_loop::EventLoop;

/// Entry point for the engine. Construct with `App::new`, configure the scene
/// via `engine_context_mut`, then call `run` to enter the event loop.
pub struct App {
    event_loop: EventLoop<()>,
    engine_context: EngineContext,
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

impl App {
    pub fn new() -> Self {
        let engine_context = EngineContext::new();
        let event_loop = EventLoop::new().expect("Failed to create event loop");
        Self { event_loop, engine_context }
    }

    /// Returns a mutable reference to the `EngineContext` for scene setup.
    pub fn engine_context_mut(&mut self) -> &mut EngineContext {
        &mut self.engine_context
    }

    /// Starts the winit event loop. Blocks until the window is closed.
    pub fn run(self) {
        let mut handler = AppHandler::new(self.engine_context);
        self.event_loop
            .run_app(&mut handler)
            .expect("Failed to run event loop");
    }
}
