use crate::device::{KeyCode, MouseButton};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct InputAction(String);

impl InputAction {
    pub fn new(name: impl Into<String>) -> Self {
        Self(name.into())
    }

    pub fn name(&self) -> &str {
        &self.0
    }
}
impl From<&str> for InputAction {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

impl From<String> for InputAction {
    fn from(s: String) -> Self {
        Self(s)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputBinding {
    Key(KeyCode),
    Mouse(MouseButton),
    //Gamepad
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputState {
    Pressed,
    Released,
    JustPressed,
    JustReleased,
    Idle,
}
