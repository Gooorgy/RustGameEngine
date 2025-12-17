use std::cell::RefCell;
use std::rc::Rc;
use game_object::traits::GameObject;

pub struct GameObjectRegistration {
    pub component: Rc<RefCell<dyn GameObject>>,
}

impl GameObjectRegistration {
    pub fn new(game_object: impl GameObject + 'static) -> Self {
        Self {
            component: Rc::new(RefCell::new(game_object)),
        }
    }
}
