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

/// Manages keyboard, mouse button, and axis input. Registered as a manager in
/// `EngineContext`. Call `update` once per frame before reading any state, then
/// `end_frame` after all systems have run to advance the prev-frame snapshot.
pub struct InputManager {
    input_state: GameInputState,
    config: InputConfig,
}

impl InputManager {
    /// Creates an `InputManager` with no bindings and zeroed input state.
    pub fn new() -> Self {
        Self {
            input_state: GameInputState::default(),
            config: InputConfig::default(),
        }
    }

    /// Binds one or more `InputBinding`s to a named action. Any of the bindings
    /// being held counts as the action being down.
    pub fn bind_action(&mut self, action: impl Into<InputAction>, bindings: Vec<InputBinding>) {
        let action = action.into();
        self.config.action_binding.insert(action, bindings);
    }

    /// Binds an axis to either a composite key pair or an analog source.
    pub fn bind_axis(&mut self, action: impl Into<AxisAction>, bindings: AxisBinding) {
        let action = action.into();
        self.config.axis_binding.insert(action, bindings);
    }

    // ---- Raw event handlers (called by the platform layer) ------------------

    /// Records a key-down event. Called by the winit event loop.
    pub fn on_key_pressed(&mut self, key: KeyCode) {
        self.input_state.keys_down.insert(key);
    }

    /// Records a key-up event. Called by the winit event loop.
    pub fn on_key_released(&mut self, key: KeyCode) {
        self.input_state.keys_down.remove(&key);
    }

    /// Records a mouse button down event. Called by the winit event loop.
    pub fn on_mouse_button_pressed(&mut self, button: MouseButton) {
        self.input_state.mouse_buttons_down.insert(button);
    }

    /// Records a mouse button up event. Called by the winit event loop.
    pub fn on_mouse_button_released(&mut self, button: MouseButton) {
        self.input_state.mouse_buttons_down.remove(&button);
    }

    /// Accumulates raw mouse motion delta. Called by the winit event loop.
    /// Delta is reset to zero by `end_frame`.
    pub fn on_mouse_moved(&mut self, delta_x: f32, delta_y: f32) {
        self.input_state.mouse_delta[0] += delta_x;
        self.input_state.mouse_delta[1] += delta_y;
    }

    /// Updates the absolute mouse position in window coordinates.
    pub fn on_mouse_position(&mut self, x: f32, y: f32) {
        self.input_state.mouse_position = [x, y];
    }

    /// Accumulates mouse wheel scroll. Called by the winit event loop.
    /// Value is reset to zero by `end_frame`.
    pub fn on_mouse_wheel(&mut self, delta: f32) {
        self.input_state.mouse_wheel += delta;
    }

    // ---- State accessors ----------------------------------------------------

    /// Returns the full raw input state for the current frame.
    pub fn get_input_state(&self) -> &GameInputState {
        &self.input_state
    }

    /// Returns the current value of the named axis, or 0.0 if not bound.
    pub fn get_axis(&self, axis: impl Into<AxisAction>) -> f32 {
        let axis = axis.into();
        *self.input_state.axis_values.get(&axis).unwrap_or(&0.0)
    }

    /// Returns true if the named action is currently held (JustPressed or Pressed).
    pub fn is_action_pressed(&self, action: impl Into<InputAction>) -> bool {
        let action = action.into();
        matches!(
            self.input_state.action_states.get(&action),
            Some(InputState::Pressed) | Some(InputState::JustPressed)
        )
    }

    /// Returns true only on the frame the action first became pressed.
    pub fn is_action_just_pressed(&self, action: impl Into<InputAction>) -> bool {
        let action = action.into();
        matches!(
            self.input_state.action_states.get(&action),
            Some(InputState::JustPressed)
        )
    }

    /// Returns true only on the frame the action was released.
    pub fn is_action_just_released(&self, action: impl Into<InputAction>) -> bool {
        let action = action.into();
        matches!(
            self.input_state.action_states.get(&action),
            Some(InputState::JustReleased)
        )
    }

    /// Returns true if the key is currently held down.
    pub fn is_key_down(&self, key: KeyCode) -> bool {
        self.input_state.keys_down.contains(&key)
    }

    /// Returns true only on the frame the key first became pressed.
    pub fn is_key_just_pressed(&self, key: KeyCode) -> bool {
        self.input_state.keys_down.contains(&key)
            && !self.input_state.prev_keys_down.contains(&key)
    }

    /// Returns true if the mouse button is currently held down.
    pub fn is_mouse_button_down(&self, button: MouseButton) -> bool {
        self.input_state.mouse_buttons_down.contains(&button)
    }

    /// Returns the raw mouse movement delta accumulated since the last `end_frame`.
    pub fn get_mouse_delta(&self) -> [f32; 2] {
        self.input_state.mouse_delta
    }

    /// Returns the current absolute mouse position in window coordinates.
    pub fn get_mouse_position(&self) -> [f32; 2] {
        self.input_state.mouse_position
    }

    /// Returns the mouse wheel scroll accumulated since the last `end_frame`.
    pub fn get_mouse_wheel(&self) -> f32 {
        self.input_state.mouse_wheel
    }

    // ---- Per-frame lifecycle ------------------------------------------------

    /// Recalculates action and axis states from the current raw input.
    /// Call once at the start of each frame, before any system reads input.
    pub fn update(&mut self) {
        self.update_action_states();
        self.update_axis_values();
    }

    /// Advances the prev-frame snapshot and clears per-frame accumulations
    /// (mouse delta, mouse wheel). Call after all systems have read input.
    pub fn end_frame(&mut self) {
        self.input_state.prev_keys_down = self.input_state.keys_down.clone();
        self.input_state.prev_mouse_buttons_down = self.input_state.mouse_buttons_down.clone();

        self.input_state.mouse_delta = [0.0; 2];
        self.input_state.mouse_wheel = 0.0;
    }

    // ---- Internal state machine ---------------------------------------------

    fn update_action_states(&mut self) {
        for (action_name, bindings) in &self.config.action_binding {
            let is_down = bindings.iter().any(|binding| self.is_binding_down(binding));
            let just_pressed = bindings
                .iter()
                .any(|binding| self.is_binding_just_pressed(binding));
            let just_released = bindings
                .iter()
                .any(|binding| self.is_binding_just_released(binding));

            // Priority: JustPressed > JustReleased > Pressed > Idle.
            // JustPressed wins so one-frame events are never masked by Pressed.
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
