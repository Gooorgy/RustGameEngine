use ecs::component::Component;
use crate::dynamic_aabb::shape::{Shape, ShapeId, ShapeStore};

/// Opaque handle to a registered collider. Wraps a Vec index.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ColliderId(pub u32);

impl ColliderId {
    /// Returns the raw index value. Prefer typed accessors where possible.
    pub fn raw(self) -> u32 { self.0 }
}

/// 4-byte record per registered collider.
/// Only shape_id for now; friction/restitution/layers can be added here later.
#[derive(Debug, Clone, Copy)]
pub struct ColliderData {
    pub shape_id: ShapeId,
}

/// Dense collider storage. ColliderId.0 is a direct Vec index.
pub struct ColliderStore {
    colliders: Vec<ColliderData>,
}

impl ColliderStore {
    pub fn new() -> Self {
        Self { colliders: Vec::new() }
    }

    /// Always creates a new ColliderId. Colliders are not deduplicated;
    /// only shapes are. Two cuboids at different positions are different colliders
    /// but may share a ShapeId.
    pub fn insert(&mut self, shape_id: ShapeId) -> ColliderId {
        debug_assert!(
            self.colliders.len() < u32::MAX as usize,
            "ColliderId overflow: more than u32::MAX colliders registered"
        );
        let id = ColliderId(self.colliders.len() as u32);
        self.colliders.push(ColliderData { shape_id });
        id
    }

    /// Returns the ColliderData for the given id, or None if the id is invalid.
    pub fn get(&self, id: ColliderId) -> Option<&ColliderData> {
        self.colliders.get(id.0 as usize)
    }

    /// Resolves a collider to its shape in one call. Returns None if either
    /// the collider id or the shape id is invalid.
    pub fn get_shape<'a>(&self, id: ColliderId, shapes: &'a ShapeStore) -> Option<&'a Shape> {
        self.get(id).and_then(|d| shapes.get(d.shape_id))
    }
}

impl Default for ColliderStore {
    fn default() -> Self { Self::new() }
}

/// ECS component storing a collider handle. Exactly 4 bytes in the component column.
#[derive(Clone, Copy, Debug, Component)]
pub struct ColliderComponent {
    pub id: ColliderId,
}
