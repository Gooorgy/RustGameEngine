use assets::MeshHandle;
use nalgebra_glm::Vec3;
use std::collections::HashMap;
use super::AABB;

/// Opaque handle to a registered shape. Wraps a Vec index.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ShapeId(pub u32);

impl ShapeId {
    /// Returns the raw index value. Prefer typed accessors where possible.
    pub fn raw(self) -> u32 { self.0 }
}

/// Geometric shape used for broad-phase collision.
///
/// All variants except `Mesh` can compute their own world-space AABB via
/// `Shape::compute_aabb`. Mesh shapes require pre-computed bounds; see
/// `SpatialWorld::tree_insert_raw`.
#[derive(Debug, Clone, PartialEq)]
pub enum Shape {
    /// A sphere with the given radius.
    Sphere { radius: f32 },
    /// An axis-aligned box with per-axis half-extents.
    Cuboid { half_extents: Vec3 },
    /// A capsule aligned to the Y axis. `half_height` is the half-length of the
    /// cylinder section (not including the hemisphere caps).
    Capsule { half_height: f32, radius: f32 },
    /// A triangle mesh. Cannot auto-compute its AABB; callers must provide
    /// pre-computed bounds via `SpatialWorld::tree_insert_raw`.
    Mesh { mesh_handle: MeshHandle },
}

/// Hashable key built from bit-cast floats (f32::to_bits() -> u32).
/// Safe because shape parameters are always finite; NaN is a caller bug.
/// pub(crate) - callers never construct or match on ShapeKey directly.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) enum ShapeKey {
    Sphere { radius: u32 },
    Cuboid { hx: u32, hy: u32, hz: u32 },
    Capsule { half_height: u32, radius: u32 },
    Mesh { mesh_id: u64 },
}

impl ShapeKey {
    pub(crate) fn from_shape(shape: &Shape) -> Self {
        match shape {
            Shape::Sphere { radius } =>
                ShapeKey::Sphere { radius: radius.to_bits() },
            Shape::Cuboid { half_extents } =>
                ShapeKey::Cuboid {
                    hx: half_extents.x.to_bits(),
                    hy: half_extents.y.to_bits(),
                    hz: half_extents.z.to_bits(),
                },
            Shape::Capsule { half_height, radius } =>
                ShapeKey::Capsule {
                    half_height: half_height.to_bits(),
                    radius: radius.to_bits(),
                },
            Shape::Mesh { mesh_handle } =>
                ShapeKey::Mesh { mesh_id: mesh_handle.raw() },
        }
    }
}

/// Dense, deduplicated shape storage.
/// shapes[ShapeId.0] - O(1) indexed access.
/// key_to_id - HashMap used only on insert for dedup, never on read.
pub struct ShapeStore {
    shapes: Vec<Shape>,
    key_to_id: HashMap<ShapeKey, ShapeId>,
}

impl ShapeStore {
    pub fn new() -> Self {
        Self { shapes: Vec::new(), key_to_id: HashMap::new() }
    }

    /// Returns an existing ShapeId if an identical shape was already registered.
    /// O(1) amortized.
    pub fn insert(&mut self, shape: Shape) -> ShapeId {
        let key = ShapeKey::from_shape(&shape);
        if let Some(&id) = self.key_to_id.get(&key) {
            return id;
        }
        debug_assert!(
            self.shapes.len() < u32::MAX as usize,
            "ShapeId overflow: more than u32::MAX shapes registered"
        );
        let id = ShapeId(self.shapes.len() as u32);
        self.shapes.push(shape);
        self.key_to_id.insert(key, id);
        id
    }

    pub fn get(&self, id: ShapeId) -> Option<&Shape> {
        self.shapes.get(id.0 as usize)
    }
}

impl Default for ShapeStore {
    fn default() -> Self { Self::new() }
}

impl Shape {
    /// Computes the world-space AABB of this shape given its center position.
    ///
    /// Rotation is not considered - shapes are treated as axis-aligned.
    /// This is correct for Sphere (rotation-invariant) and gives a conservative
    /// broad-phase bound for Capsule (Y-up axis assumed).
    /// For Cuboid, pass axis-aligned half_extents. If your object is rotated,
    /// pre-expand the half_extents or use `AABB::new()` directly.
    ///
    /// Panics if called on a `Mesh` variant. Use `SpatialWorld::tree_insert_raw`
    /// with pre-computed bounds for mesh colliders.
    pub fn compute_aabb(&self, center: Vec3) -> AABB {
        match self {
            Shape::Sphere { radius } => {
                let r = Vec3::new(*radius, *radius, *radius);
                AABB::new(center - r, center + r)
            }
            Shape::Cuboid { half_extents } => {
                AABB::new(center - half_extents, center + half_extents)
            }
            Shape::Capsule { half_height, radius } => {
                let half = Vec3::new(*radius, half_height + radius, *radius);
                AABB::new(center - half, center + half)
            }
            Shape::Mesh { .. } => {
                panic!(
                    "Shape::Mesh cannot compute AABB automatically. \
                     Pre-compute bounds and call tree_insert_raw() instead."
                )
            }
        }
    }
}
