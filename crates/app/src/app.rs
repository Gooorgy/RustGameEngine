use crate::app_handler::AppHandler;
use asset_pipeline::cook_pending;
use config::config::ConfigFile;
use core::asset_context::AssetContext;
use core::{EngineConfig, EngineContext};
use project::{AssetRegistry, Project};
use winit::event_loop::EventLoop;

/// Entry point for the engine. Construct with `App::new` or `App::with_project`,
/// configure the scene via `engine_context_mut`, then call `run`.
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
    /// Discovers the `.eproj` file in the current directory and loads the project.
    pub fn new() -> Self {
        let path = find_project_file().expect("no .eproj file found in the current directory");
        Self::with_project(path)
    }

    /// Loads a project from an explicit `.eproj` path.
    pub fn with_project(path: impl AsRef<std::path::Path>) -> Self {
        let path = path.as_ref();
        let project = Project::load(path)
            .unwrap_or_else(|e| panic!("failed to load '{}': {}", path.display(), e));

        let registry = AssetRegistry::load_or_scan(&project.cache_dir, &project.content_dir)
            .expect("failed to scan project content directory");

        cook_pending(&registry, &project.cache_dir, &project.content_dir);

        registry
            .save(&project.cache_dir)
            .unwrap_or_else(|e| eprintln!("warning: could not save asset registry: {}", e));

        let cfg = ConfigFile::load_or_default(&project.name);

        let config = EngineConfig {
            name: project.name,
            content_dir: project.content_dir.clone(),
            cache_dir: project.cache_dir.clone(),
            window_resolution: cfg.graphics_settings.resolution_settings,
            window_mode: cfg.graphics_settings.window_mode,
        };

        let assets = AssetContext::new(project.cache_dir, project.content_dir, registry);
        let engine_context = EngineContext::new(config, assets);
        let event_loop = EventLoop::new().expect("failed to create event loop");

        Self { event_loop, engine_context }
    }

    pub fn engine_context_mut(&mut self) -> &mut EngineContext {
        &mut self.engine_context
    }

    pub fn run(self) {
        let mut handler = AppHandler::new(self.engine_context);
        self.event_loop
            .run_app(&mut handler)
            .expect("Failed to run event loop");
    }
}

fn find_project_file() -> Option<std::path::PathBuf> {
    std::fs::read_dir(".")
        .ok()?
        .filter_map(|e| e.ok())
        .find(|e| e.path().extension().and_then(|x| x.to_str()) == Some("eproj"))
        .map(|e| e.path())
}