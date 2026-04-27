use crate::app_handler::AppHandler;
use asset_pipeline::cook_pending;
use core::EngineContext;
use project::{AssetRegistry, Project};
use std::path::PathBuf;
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
    /// Discovers the `.eproj` file in the current directory, loads the project,
    /// scans the content directory to build the asset registry, then initialises
    /// the engine context.
    pub fn new() -> Self {
        let project_path = find_project_file()
            .expect("no .eproj file found in the current directory");
        let project = Project::load(&project_path)
            .unwrap_or_else(|e| panic!("failed to load '{}': {}", project_path.display(), e));
        let registry = AssetRegistry::scan(&project, None)
            .expect("failed to scan project content directory");
        cook_pending(&registry, &project);
        registry.save(&project)
            .unwrap_or_else(|e| eprintln!("warning: could not save asset registry: {}", e));

        let engine_context = EngineContext::new(project, registry);
        let event_loop = EventLoop::new().expect("failed to create event loop");
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

fn find_project_file() -> Option<PathBuf> {
    std::fs::read_dir(".")
        .ok()?
        .filter_map(|e| e.ok())
        .find(|e| {
            e.path()
                .extension()
                .and_then(|x| x.to_str())
                == Some("eproj")
        })
        .map(|e| e.path())
}
