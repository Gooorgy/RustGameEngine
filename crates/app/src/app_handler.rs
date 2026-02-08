use assets::AssetManager;
use core::{CameraControllerComponent, EngineContext, TransformComponent};
use ecs::world::World;
use input::InputManager;
use material::material_manager::MaterialManager;
use renderer::frame_data::{Resolution, ResolutionSettings};
use renderer::render_data::RenderDataCollector;
use renderer::renderer::Renderer;
use rendering_backend::backend_impl::resource_manager::ResourceManager;
use rendering_backend::backend_impl::vulkan_backend::VulkanBackend;
use rendering_backend::camera::CameraMvpUbo;
use std::cell::RefMut;
use std::io::Write;
use std::time::Instant;
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::error::OsError;
use winit::event::{DeviceEvent, DeviceId, ElementState, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::KeyCode as WinitKeyCode;
use winit::window::{Window, WindowId};

// Replace this with env lookup?
const WINDOW_TITLE: &str = "Vulkan Test";
const WINDOW_WIDTH: u32 = 800;
const WINDOW_HEIGHT: u32 = 600;

pub struct AppHandler {
    engine_context: EngineContext,
    window: Option<Window>,
    vulkan_backend: Option<VulkanBackend>,
    renderer: Option<Renderer>,
    last_frame_time: Instant,
}

impl AppHandler {
    pub fn new(engine_context: EngineContext) -> Self {
        Self {
            window: None,
            engine_context,
            vulkan_backend: None,
            renderer: None,
            last_frame_time: Instant::now(),
        }
    }

    fn create_window(&mut self, event_loop: &ActiveEventLoop) -> Result<Window, OsError> {
        let window_attributes = Window::default_attributes()
            .with_title(WINDOW_TITLE)
            .with_inner_size(LogicalSize::new(WINDOW_WIDTH, WINDOW_HEIGHT));

        event_loop.create_window(window_attributes)
    }

    pub fn get_from_context<T: 'static>(&self) -> RefMut<T> {
        self.engine_context.expect_manager_mut::<T>()
    }

    pub fn get_world(&mut self) -> &mut World {
        self.engine_context.get_world()
    }
}

impl ApplicationHandler for AppHandler {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_none() {
            self.window = Some(
                self.create_window(event_loop)
                    .expect("Failed to create window"),
            );

            let mut vulkan = VulkanBackend::new(self.window.as_ref().unwrap()).expect("");

            // TODO: This is bad. but works for now...

            let mut renderer = Renderer::new(
                &mut vulkan,
                ResolutionSettings {
                    window_resolution: Resolution {
                        height: WINDOW_HEIGHT,
                        width: WINDOW_WIDTH,
                    },
                    shadow_resolution: Resolution {
                        width: 1024,
                        height: 1024,
                    },
                },
            );

            self.renderer = Some(renderer);
            self.vulkan_backend = Some(vulkan);
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
            WindowEvent::RedrawRequested => match self.vulkan_backend {
                Some(ref mut app) => {
                    let time_elapsed = self.last_frame_time.elapsed();
                    self.last_frame_time = Instant::now();
                    let delta_time = time_elapsed.subsec_micros() as f32 / 1_000_000.0_f32;
                    std::io::stdout().flush().unwrap();

                    {
                        let mut input_manager =
                            self.engine_context.expect_manager_mut::<InputManager>();
                        input_manager.update();

                        input_manager.end_frame();
                        drop(input_manager);

                        // Update camera controller via ECS
                        self.engine_context.update(delta_time);
                    }

                    let world = self.engine_context.get_world();
                    let mut render_data = RenderDataCollector::new();
                    render_data
                        .collect_from_world(world, WINDOW_WIDTH as f32 / WINDOW_HEIGHT as f32);

                    let camera_ubo = render_data
                        .camera
                        .map(|c| CameraMvpUbo {
                            view: c.view,
                            proj: c.proj,
                        })
                        .expect("No active camera in world");

                    let mut asset_manager =
                        self.engine_context.expect_manager_mut::<AssetManager>();
                    let mut material_manager =
                        self.engine_context.expect_manager_mut::<MaterialManager>();
                    let mut resource_manager =
                        self.engine_context.expect_manager_mut::<ResourceManager>();

                    let renderer = self.renderer.as_mut().unwrap();

                    renderer.draw_frame(
                        app,
                        &render_data.mesh_requests,
                        &mut material_manager,
                        &mut asset_manager,
                        &mut resource_manager,
                        camera_ubo,
                    );

                    let window = &self.window.as_ref().unwrap();
                    window.set_title(&format!(
                        "{} - FPS: {}- FrameTime: {}",
                        WINDOW_TITLE,
                        1f32 / delta_time,
                        delta_time
                    ));

                    Window::request_redraw(window);
                }
                _ => panic!("Vulkan backend not initialized"),
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
        let mut input_manager = self.engine_context.expect_manager_mut::<InputManager>();

        match event {
            DeviceEvent::MouseMotion { delta } => {
                input_manager.on_mouse_moved(delta.0 as f32, delta.1 as f32);
            }
            DeviceEvent::Key(raw_key_event) => {
                if let winit::keyboard::PhysicalKey::Code(key_code) = raw_key_event.physical_key {
                    if let Some(input_key) = convert_winit_keycode(key_code) {
                        match raw_key_event.state {
                            ElementState::Pressed => input_manager.on_key_pressed(input_key),
                            ElementState::Released => input_manager.on_key_released(input_key),
                        }
                    }
                }
            }
            DeviceEvent::MouseWheel { delta } => {
                let scroll = match delta {
                    winit::event::MouseScrollDelta::LineDelta(_, y) => y,
                    winit::event::MouseScrollDelta::PixelDelta(pos) => pos.y as f32 / 100.0,
                };
                input_manager.on_mouse_wheel(scroll);
            }
            DeviceEvent::Button { button, state } => {
                let mouse_button = match button {
                    0 => Some(input::MouseButton::Left),
                    1 => Some(input::MouseButton::Right),
                    2 => Some(input::MouseButton::Middle),
                    3 => Some(input::MouseButton::Button4),
                    4 => Some(input::MouseButton::Button5),
                    _ => None,
                };
                if let Some(btn) = mouse_button {
                    match state {
                        ElementState::Pressed => input_manager.on_mouse_button_pressed(btn),
                        ElementState::Released => input_manager.on_mouse_button_released(btn),
                    }
                }
            }
            _ => {}
        }
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
