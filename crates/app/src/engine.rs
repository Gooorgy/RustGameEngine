use core::EngineContext;
use renderer::frame_data::{Resolution, ResolutionSettings};
use renderer::render_data::RenderDataCollector;
use renderer::renderer::{DebugBox, Renderer};
use rendering_backend::backend_impl::resource_manager::ResourceManager;
use rendering_backend::backend_impl::vulkan_backend::VulkanBackend;
use rendering_backend::camera::CameraMvpUbo;
use std::time::Instant;
use winit::event::{DeviceEvent, ElementState};
use winit::keyboard::KeyCode as WinitKeyCode;
use winit::window::Window;

const WINDOW_TITLE: &str = "Vulkan Test";
/// How often the title bar FPS/frametime counters are refreshed (seconds).
const DISPLAY_INTERVAL: f32 = 0.5;

pub(crate) struct Engine {
    context: EngineContext,
    vulkan_backend: VulkanBackend,
    resource_manager: ResourceManager,
    renderer: Renderer,
    window: Window,
    last_frame_time: Instant,
    frame_time_accum: f32,
    frame_count: u32,
    displayed_fps: f32,
    displayed_ms: f32,
}

impl Engine {
    /// Initialises Vulkan and the renderer, then takes ownership of the pre-configured context.
    pub fn new(window: Window, context: EngineContext) -> Self {
        let size = window.inner_size();
        let mut vulkan_backend =
            VulkanBackend::new(&window).expect("Failed to initialize Vulkan backend");

        let renderer = Renderer::new(
            &mut vulkan_backend,
            ResolutionSettings {
                window_resolution: Resolution {
                    width: size.width,
                    height: size.height,
                },
                shadow_resolutions: vec![
                    Resolution { width: 2048, height: 2048 },
                    Resolution { width: 2048, height: 2048 },
                    Resolution { width: 1024, height: 1024 },
                    Resolution { width: 1024, height: 1024 },
                ],
            },
        );

        Self {
            context,
            vulkan_backend,
            resource_manager: ResourceManager::new(),
            renderer,
            window,
            last_frame_time: Instant::now(),
            frame_time_accum: 0.0,
            frame_count: 0,
            displayed_fps: 0.0,
            displayed_ms: 0.0,
        }
    }

    /// Runs one full engine frame: input → ECS → render → present.
    pub fn tick(&mut self) {
        let delta_time = self.last_frame_time.elapsed().as_secs_f32();
        self.last_frame_time = Instant::now();

        {
            let mut input = self.context.input();
            input.update();
            if input.is_key_just_pressed(input::KeyCode::F3) {
                self.renderer.toggle_aabb_debug();
            }
            input.end_frame();
        }

        self.context.update(delta_time);

        let size = self.window.inner_size();
        let aspect = size.width as f32 / size.height as f32;

        let world = self.context.get_world();
        let mut render_data = RenderDataCollector::new();
        render_data.collect_from_world(world, aspect);

        let camera_render_data = render_data.camera;
        let camera_ubo = camera_render_data
            .as_ref()
            .map(|c| CameraMvpUbo { view: c.view, proj: c.proj })
            .expect("No active camera in world");

        let directional_light = render_data.directional_light;

        let mut asset_manager = self.context.assets();
        let mut material_manager = self.context.materials();

        let debug_boxes = self
            .context
            .get_spatial_world()
            .iter_aabbs()
            .map(|aabb| DebugBox { max: aabb.upper, min: aabb.lower })
            .collect::<Vec<_>>();

        self.renderer.draw_frame(
            &mut self.vulkan_backend,
            &render_data.mesh_requests,
            &mut material_manager,
            &mut asset_manager,
            &mut self.resource_manager,
            camera_ubo,
            camera_render_data,
            directional_light,
            &debug_boxes,
        );

        // Accumulate frame time; update the displayed values every DISPLAY_INTERVAL seconds
        // so the title counter is stable and readable rather than flipping every frame.
        self.frame_time_accum += delta_time;
        self.frame_count += 1;

        if self.frame_time_accum >= DISPLAY_INTERVAL {
            self.displayed_fps = self.frame_count as f32 / self.frame_time_accum;
            self.displayed_ms = (self.frame_time_accum / self.frame_count as f32) * 1000.0;
            self.frame_time_accum = 0.0;
            self.frame_count = 0;
        }

        self.window.set_title(&format!(
            "{} - FPS: {:.0} - FrameTime: {:.2}ms",
            WINDOW_TITLE, self.displayed_fps, self.displayed_ms
        ));

        self.window.request_redraw();
    }

    /// Forwards a winit device event to the input manager.
    pub fn handle_device_event(&mut self, event: DeviceEvent) {
        let mut input = self.context.input();

        match event {
            DeviceEvent::MouseMotion { delta } => {
                input.on_mouse_moved(delta.0 as f32, delta.1 as f32);
            }
            DeviceEvent::Key(raw) => {
                if let winit::keyboard::PhysicalKey::Code(key_code) = raw.physical_key {
                    if let Some(key) = convert_winit_keycode(key_code) {
                        match raw.state {
                            ElementState::Pressed => input.on_key_pressed(key),
                            ElementState::Released => input.on_key_released(key),
                        }
                    }
                }
            }
            DeviceEvent::MouseWheel { delta } => {
                let scroll = match delta {
                    winit::event::MouseScrollDelta::LineDelta(_, y) => y,
                    winit::event::MouseScrollDelta::PixelDelta(pos) => pos.y as f32 / 100.0,
                };
                input.on_mouse_wheel(scroll);
            }
            DeviceEvent::Button { button, state } => {
                let btn = match button {
                    0 => Some(input::MouseButton::Left),
                    1 => Some(input::MouseButton::Right),
                    2 => Some(input::MouseButton::Middle),
                    3 => Some(input::MouseButton::Button4),
                    4 => Some(input::MouseButton::Button5),
                    _ => None,
                };
                if let Some(btn) = btn {
                    match state {
                        ElementState::Pressed => input.on_mouse_button_pressed(btn),
                        ElementState::Released => input.on_mouse_button_released(btn),
                    }
                }
            }
            _ => {}
        }
    }

    #[allow(dead_code)]
    pub fn window(&self) -> &Window {
        &self.window
    }
}

fn convert_winit_keycode(key: WinitKeyCode) -> Option<input::KeyCode> {
    use input::KeyCode;
    Some(match key {
        WinitKeyCode::KeyA => KeyCode::A,
        WinitKeyCode::KeyB => KeyCode::B,
        WinitKeyCode::KeyC => KeyCode::C,
        WinitKeyCode::KeyD => KeyCode::D,
        WinitKeyCode::KeyE => KeyCode::E,
        WinitKeyCode::KeyF => KeyCode::F,
        WinitKeyCode::KeyG => KeyCode::G,
        WinitKeyCode::KeyH => KeyCode::H,
        WinitKeyCode::KeyI => KeyCode::I,
        WinitKeyCode::KeyJ => KeyCode::J,
        WinitKeyCode::KeyK => KeyCode::K,
        WinitKeyCode::KeyL => KeyCode::L,
        WinitKeyCode::KeyM => KeyCode::M,
        WinitKeyCode::KeyN => KeyCode::N,
        WinitKeyCode::KeyO => KeyCode::O,
        WinitKeyCode::KeyP => KeyCode::P,
        WinitKeyCode::KeyQ => KeyCode::Q,
        WinitKeyCode::KeyR => KeyCode::R,
        WinitKeyCode::KeyS => KeyCode::S,
        WinitKeyCode::KeyT => KeyCode::T,
        WinitKeyCode::KeyU => KeyCode::U,
        WinitKeyCode::KeyV => KeyCode::V,
        WinitKeyCode::KeyW => KeyCode::W,
        WinitKeyCode::KeyX => KeyCode::X,
        WinitKeyCode::KeyY => KeyCode::Y,
        WinitKeyCode::KeyZ => KeyCode::Z,
        WinitKeyCode::Digit0 => KeyCode::Key0,
        WinitKeyCode::Digit1 => KeyCode::Key1,
        WinitKeyCode::Digit2 => KeyCode::Key2,
        WinitKeyCode::Digit3 => KeyCode::Key3,
        WinitKeyCode::Digit4 => KeyCode::Key4,
        WinitKeyCode::Digit5 => KeyCode::Key5,
        WinitKeyCode::Digit6 => KeyCode::Key6,
        WinitKeyCode::Digit7 => KeyCode::Key7,
        WinitKeyCode::Digit8 => KeyCode::Key8,
        WinitKeyCode::Digit9 => KeyCode::Key9,
        WinitKeyCode::Space => KeyCode::Space,
        WinitKeyCode::Enter => KeyCode::Enter,
        WinitKeyCode::Tab => KeyCode::Tab,
        WinitKeyCode::Backspace => KeyCode::Backspace,
        WinitKeyCode::Delete => KeyCode::Delete,
        WinitKeyCode::Escape => KeyCode::Escape,
        WinitKeyCode::ShiftLeft | WinitKeyCode::ShiftRight => KeyCode::Shift,
        WinitKeyCode::ControlLeft | WinitKeyCode::ControlRight => KeyCode::Control,
        WinitKeyCode::AltLeft | WinitKeyCode::AltRight => KeyCode::Alt,
        WinitKeyCode::ArrowLeft => KeyCode::Left,
        WinitKeyCode::ArrowRight => KeyCode::Right,
        WinitKeyCode::ArrowUp => KeyCode::Up,
        WinitKeyCode::ArrowDown => KeyCode::Down,
        WinitKeyCode::F1 => KeyCode::F1,
        WinitKeyCode::F2 => KeyCode::F2,
        WinitKeyCode::F3 => KeyCode::F3,
        WinitKeyCode::F4 => KeyCode::F4,
        WinitKeyCode::F5 => KeyCode::F5,
        WinitKeyCode::F6 => KeyCode::F6,
        WinitKeyCode::F7 => KeyCode::F7,
        WinitKeyCode::F8 => KeyCode::F8,
        WinitKeyCode::F9 => KeyCode::F9,
        WinitKeyCode::F10 => KeyCode::F10,
        WinitKeyCode::F11 => KeyCode::F11,
        WinitKeyCode::F12 => KeyCode::F12,
        _ => return None,
    })
}
