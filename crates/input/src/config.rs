use std::collections::HashMap;
use crate::axis_action::{AxisAction, AxisBinding};
use crate::input_action::{InputAction, InputBinding};

#[derive(Debug, Default)]
pub struct InputConfig {
    pub action_binding: HashMap<InputAction, Vec<InputBinding>>,
    pub axis_binding: HashMap<AxisAction, AxisBinding>,
}