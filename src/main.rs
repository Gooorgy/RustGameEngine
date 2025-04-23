use app::App;
use assets::AssetManager;
use core::EngineContext;
use scene::{SceneManager, StaticMesh};

const WINDOW_TITLE: &str = "Vulkan Test";
const WINDOW_WIDTH: u32 = 800;
const WINDOW_HEIGHT: u32 = 600;
/*struct AppWindow {
    window: Option<Window>,
    vulkan_app: Option<VulkanBackend>,
    last_frame_time: Instant,
    app: App,
}

impl AppWindow {}

impl Default for AppWindow {
    fn default() -> Self {
        /*        let scene_root = SceneNode::new(
            ".\\resources\\models\\test.obj",
            ".\\resources\\textures\\texture.png",
        );*/
        //scene_root.borrow_mut().add_child("E:\\rust\\new\\src\\models\\test2.obj");

        let block_registry: HashMap<BlockNameSpace, BlockDefinition> = HashMap::from([
            (BlockType::GRASS.as_namespace(), GRASS),
            (BlockType::STONE.as_namespace(), STONE),
            (BlockType::NONE.as_namespace(), NONE),
        ]);

        let mut terrain = Terrain::new(block_registry);
        terrain.add_chunk();

        //let mesh = terrain.get_chuck(0).build_chunk_mesh();

        let mut app = App::new();
        let material = Material::pbr("TestMaterial",PbrMaterialInstance::default().with_albedo_texture(".\\resources\\textures\\texture.png"));

        let static_mesh = StaticMesh::new(String::from(".\\resources\\models\\test.obj"));
        app.register_component(static_mesh);
        let mesh2 = StaticMesh::new(String::from(".\\resources\\models\\untitled.obj"));
        app.register_component(mesh2);

        Self {
            window: None,
            vulkan_app: None,
            last_frame_time: Instant::now(),
            app,
        }
    }
}

impl ApplicationHandler for AppWindow {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window_attributes = Window::default_attributes()
            .with_title(WINDOW_TITLE)
            .with_inner_size(LogicalSize::new(WINDOW_WIDTH, WINDOW_HEIGHT));
        self.window = Some(event_loop.create_window(window_attributes).unwrap());

        let mut vulkan = VulkanBackend::new(self.window.as_ref().unwrap()).expect("");

        self.app.init();
        let mesh_assets = self
            .app
            .get_static_meshes()
            .iter()
            .map(|asse| asse.data.mesh.clone())
            .collect::<Vec<_>>();
        vulkan.upload_meshes(mesh_assets, Transform::default().model);

        self.vulkan_app = Some(vulkan);
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
                _ => panic!(""),
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
        let vulkan_app = self.vulkan_app.as_mut().unwrap();
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
*/
fn main() {
    let mut engine_context = EngineContext::new();
    let scene_manager = SceneManager::new();
    let asset_manager = AssetManager::default();

    engine_context.insert(scene_manager);
    engine_context.insert(asset_manager);

    let mut app = App::new(engine_context);

    let static_mesh = StaticMesh::new(String::from(".\\resources\\models\\test.obj"));

    app.get_from_context::<SceneManager>()
        .unwrap()
        .register_component(static_mesh);

    app.run();
}
