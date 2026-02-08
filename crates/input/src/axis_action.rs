use crate::input_action::InputAction;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AxisAction(String);

impl AxisAction {
    pub fn new(name: impl Into<String>) -> Self {
        Self(name.into())
    }

    pub fn name(&self) -> &str {
        &self.0
    }

    pub const HORIZONTAL: &'static str = "horizontal";
    pub const VERTICAL: &'static str = "vertical";
    pub const MOUSE_X: &'static str = "mouse_x";
    pub const MOUSE_Y: &'static str = "mouse_y";
    pub const MOUSE_WHEEL: &'static str = "mouse_wheel";
}

impl From<&str> for AxisAction {
    fn from(s: &str) -> Self {
        Self(s.to_string())
    }
}

#[derive(Debug, Clone)]
pub enum AxisBinding {
    Composite {
        positive: InputAction,
        negative: InputAction,
    },
    Analog {
        source: AnalogSource,
        sensitivity: f32,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnalogSource {
    MouseX,
    MouseY,
    MouseWheel,
    //GamepadAxis,
}
