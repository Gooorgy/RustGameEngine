use crate::AppHandler::AppHandler;
use core::EngineContext;
use winit::event_loop::EventLoop;

pub struct App {
    event_loop: EventLoop<()>,
    app_handler: AppHandler,
}

impl App {
    pub fn new(engine_context: EngineContext) -> App {
        let event_loop = EventLoop::new().expect("Failed to create event loop");

        let app_handler = AppHandler::new(engine_context);

        Self {
            event_loop,
            app_handler,
        }
    }

    pub fn run(mut self) {        
        self.event_loop
            .run_app(&mut self.app_handler)
            .expect("Failed to run event loop");
    }

    pub fn get_from_context<T: 'static>(&mut self) -> Option<&mut T> {
        self.app_handler.get_from_context::<T>()
    }
}
