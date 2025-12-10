use crate::Component;

pub struct ResolvedHandle<T: Component> {
    component: T,
}
