mod dynamic_aabb;

pub use dynamic_aabb::collider::{ColliderId, ColliderComponent};
pub use dynamic_aabb::shape::{Shape, ShapeId};
pub use dynamic_aabb::AABB;

use nalgebra_glm::Vec3;

use dynamic_aabb::collider::ColliderStore;
use dynamic_aabb::shape::ShapeStore;
use dynamic_aabb::DynamicAABBTree;

/// Top-level broad-phase collision world.
///
/// Owns shape and collider storage plus the underlying `DynamicAABBTree`.
/// The typical per-frame loop is:
/// 1. Call `clear_tree` to reset the tree.
/// 2. For each entity with a collider and a transform, call `insert_collider`.
/// 3. Query `iter_aabbs` for debug visualisation or downstream checks.
pub struct SpatialWorld {
    shape_store: ShapeStore,
    collider_store: ColliderStore,
    tree: DynamicAABBTree,
}

impl SpatialWorld {
    /// Creates an empty spatial world with no shapes, colliders, or tree nodes.
    pub fn new() -> Self {
        Self {
            shape_store: ShapeStore::new(),
            collider_store: ColliderStore::new(),
            tree: DynamicAABBTree::default(),
        }
    }

    // ---- Shape API ----------------------------------------------------------

    /// Registers a shape and returns its id. Shapes with identical parameters
    /// return the same id. Use this when you need to share one shape across
    /// many colliders explicitly.
    pub fn register_shape(&mut self, shape: Shape) -> ShapeId {
        self.shape_store.insert(shape)
    }

    /// Returns the shape for the given id, or None if the id is invalid.
    pub fn get_shape(&self, id: ShapeId) -> Option<&Shape> {
        self.shape_store.get(id)
    }

    // ---- Collider API -------------------------------------------------------

    /// Registers a collider by passing the shape directly. Shape deduplication
    /// is automatic - identical shape parameters reuse the same ShapeId internally.
    pub fn register_collider(&mut self, shape: Shape) -> ColliderId {
        let shape_id = self.shape_store.insert(shape);
        self.collider_store.insert(shape_id)
    }

    /// Registers a collider using a ShapeId you already hold. Use this when you
    /// explicitly manage shape sharing across many colliders.
    pub fn register_collider_shared(&mut self, shape_id: ShapeId) -> ColliderId {
        self.collider_store.insert(shape_id)
    }

    /// Resolves a collider to its shape in one call (two Vec index ops, O(1)).
    /// Returns None if the id is invalid.
    pub fn get_collider_shape(&self, id: ColliderId) -> Option<&Shape> {
        self.collider_store.get_shape(id, &self.shape_store)
    }

    // ---- Tree API -----------------------------------------------------------

    /// Clears all nodes from the broad-phase tree. Call this at the start of
    /// each frame before re-inserting colliders.
    pub fn clear_tree(&mut self) {
        self.tree.clear();
    }

    /// Inserts a registered collider into the broad-phase tree.
    /// Looks up the shape, computes the world-space AABB from `center`,
    /// then inserts the leaf. Two Vec index ops plus O(log n) tree insert.
    ///
    /// Panics if `id` is invalid or the collider's shape is a Mesh variant.
    /// Use `tree_insert_raw` for mesh colliders with pre-computed bounds.
    pub fn insert_collider(&mut self, id: ColliderId, center: Vec3) {
        let shape = self
            .collider_store
            .get_shape(id, &self.shape_store)
            .expect("insert_collider: ColliderId not found");
        let aabb = shape.compute_aabb(center);
        self.tree.insert_leaf(aabb, id);
    }

    /// Inserts a collider with a caller-supplied AABB. Use for mesh colliders
    /// or when you want to include rotation in the bounds.
    pub fn tree_insert_raw(&mut self, id: ColliderId, aabb: AABB) {
        self.tree.insert_leaf(aabb, id);
    }

    /// Returns an iterator over the AABBs of all nodes in the tree, including
    /// internal nodes. Internal node AABBs are union bounds over their subtree,
    /// which can be useful for visualising tree structure.
    pub fn iter_aabbs(&self) -> impl Iterator<Item = &AABB> {
        self.tree.aabbs()
    }
}

impl Default for SpatialWorld {
    fn default() -> Self { Self::new() }
}
