use crate::axis_action::{AnalogSource, AxisAction, AxisBinding};
use crate::config::InputConfig;
use crate::device::{KeyCode, MouseButton};
use crate::input_action::{InputAction, InputBinding, InputState};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Default)]
pub struct GameInputState {
    keys_down: HashSet<KeyCode>,
    prev_keys_down: HashSet<KeyCode>,

    mouse_buttons_down: HashSet<MouseButton>,
    prev_mouse_buttons_down: HashSet<MouseButton>,

    action_states: HashMap<InputAction, InputState>,
    axis_values: HashMap<AxisAction, f32>,

    mouse_position: [f32; 2],
    mouse_delta: [f32; 2],
    mouse_wheel: f32,
}

pub struct InputManager {
    input_state: GameInputState,
    config: InputConfig,
}

impl InputManager {
    pub fn new() -> Self {
        Self {
            input_state: GameInputState::default(),
            config: InputConfig::default(),
        }
    }

    pub fn bind_action(&mut self, action: impl Into<InputAction>, bindings: Vec<InputBinding>) {
        let action = action.into();
        self.config.action_binding.insert(action, bindings);
    }

    pub fn bind_axis(&mut self, action: impl Into<AxisAction>, bindings: AxisBinding) {
        let action = action.into();
        self.config.axis_binding.insert(action, bindings);
    }

    pub fn on_key_pressed(&mut self, key: KeyCode) {
        self.input_state.keys_down.insert(key);
    }

    pub fn on_key_released(&mut self, key: KeyCode) {
        self.input_state.keys_down.remove(&key);
    }

    pub fn on_mouse_button_pressed(&mut self, button: MouseButton) {
        self.input_state.mouse_buttons_down.insert(button);
    }

    pub fn on_mouse_button_released(&mut self, button: MouseButton) {
        self.input_state.mouse_buttons_down.remove(&button);
    }

    pub fn on_mouse_moved(&mut self, delta_x: f32, delta_y: f32) {
        self.input_state.mouse_delta[0] += delta_x;
        self.input_state.mouse_delta[1] += delta_y;
    }

    pub fn on_mouse_position(&mut self, x: f32, y: f32) {
        self.input_state.mouse_position = [x, y];
    }

    pub fn on_mouse_wheel(&mut self, delta: f32) {
        self.input_state.mouse_wheel += delta;
    }

    pub fn get_input_state(&self) -> &GameInputState {
        &self.input_state
    }

    pub fn get_axis(&self, axis: impl Into<AxisAction>) -> f32 {
        let axis = axis.into();
        *self.input_state.axis_values.get(&axis).unwrap_or(&0.0)
    }

    pub fn is_action_pressed(&self, action: impl Into<InputAction>) -> bool {
        let action = action.into();
        matches!(
            self.input_state.action_states.get(&action),
            Some(InputState::Pressed) | Some(InputState::JustPressed)
        )
    }

    pub fn is_action_just_pressed(&self, action: impl Into<InputAction>) -> bool {
        let action = action.into();
        matches!(
            self.input_state.action_states.get(&action),
            Some(InputState::JustPressed)
        )
    }

    pub fn is_action_just_released(&self, action: impl Into<InputAction>) -> bool {
        let action = action.into();
        matches!(
            self.input_state.action_states.get(&action),
            Some(InputState::JustReleased)
        )
    }

    pub fn is_key_down(&self, key: KeyCode) -> bool {
        self.input_state.keys_down.contains(&key)
    }

    pub fn is_mouse_button_down(&self, button: MouseButton) -> bool {
        self.input_state.mouse_buttons_down.contains(&button)
    }

    pub fn get_mouse_delta(&self) -> [f32; 2] {
        self.input_state.mouse_delta
    }

    pub fn get_mouse_position(&self) -> [f32; 2] {
        self.input_state.mouse_position
    }

    pub fn get_mouse_wheel(&self) -> f32 {
        self.input_state.mouse_wheel
    }

    pub fn update(&mut self) {
        self.update_action_states();
        self.update_axis_values();
    }

    pub fn end_frame(&mut self) {
        self.input_state.prev_keys_down = self.input_state.keys_down.clone();
        self.input_state.prev_mouse_buttons_down = self.input_state.mouse_buttons_down.clone();

        self.input_state.mouse_delta = [0.0; 2];
        self.input_state.mouse_wheel = 0.0;
    }

    fn update_action_states(&mut self) {
        for (action_name, bindings) in &self.config.action_binding {
            let is_down = bindings.iter().any(|binding| self.is_binding_down(binding));
            let just_pressed = bindings
                .iter()
                .any(|binding| self.is_binding_just_pressed(binding));
            let just_released = bindings
                .iter()
                .any(|binding| self.is_binding_just_released(binding));

            let new_state = if just_pressed {
                InputState::JustPressed
            } else if just_released {
                InputState::JustReleased
            } else if is_down {
                InputState::Pressed
            } else {
                InputState::Idle
            };

            self.input_state
                .action_states
                .insert(action_name.clone(), new_state);
        }
    }

    fn update_axis_values(&mut self) {
        let axis_values: Vec<(AxisAction, f32)> = self
            .config
            .axis_binding
            .iter()
            .map(|(axis_name, binding)| {
                let value = match binding {
                    AxisBinding::Composite { positive, negative } => {
                        let pos_state = self.input_state.action_states.get(positive);
                        let neg_state = self.input_state.action_states.get(negative);

                        let pos_value = match pos_state {
                            Some(InputState::Pressed) | Some(InputState::JustPressed) => 1.0,
                            _ => 0.0,
                        };
                        let neg_value = match neg_state {
                            Some(InputState::Pressed) | Some(InputState::JustPressed) => 1.0,
                            _ => 0.0,
                        };

                        pos_value - neg_value
                    }
                    AxisBinding::Analog {
                        source,
                        sensitivity,
                    } => {
                        let raw_value = match source {
                            AnalogSource::MouseX => self.input_state.mouse_delta[0],
                            AnalogSource::MouseY => self.input_state.mouse_delta[1],
                            AnalogSource::MouseWheel => self.input_state.mouse_wheel,
                        };
                        raw_value * sensitivity
                    }
                };
                (axis_name.clone(), value)
            })
            .collect();

        for (axis_name, value) in axis_values {
            self.input_state.axis_values.insert(axis_name, value);
        }
    }

    fn is_binding_down(&self, binding: &InputBinding) -> bool {
        match binding {
            InputBinding::Key(key) => self.input_state.keys_down.contains(key),
            InputBinding::Mouse(button) => self.input_state.mouse_buttons_down.contains(button),
        }
    }

    fn is_binding_just_pressed(&self, binding: &InputBinding) -> bool {
        match binding {
            InputBinding::Key(key) => {
                self.input_state.keys_down.contains(key)
                    && !self.input_state.prev_keys_down.contains(key)
            }
            InputBinding::Mouse(button) => {
                self.input_state.mouse_buttons_down.contains(button)
                    && !self.input_state.prev_mouse_buttons_down.contains(button)
            }
        }
    }

    fn is_binding_just_released(&self, binding: &InputBinding) -> bool {
        match binding {
            InputBinding::Key(key) => {
                !self.input_state.keys_down.contains(key)
                    && self.input_state.prev_keys_down.contains(key)
            }
            InputBinding::Mouse(button) => {
                !self.input_state.mouse_buttons_down.contains(button)
                    && self.input_state.prev_mouse_buttons_down.contains(button)
            }
        }
    }
}
