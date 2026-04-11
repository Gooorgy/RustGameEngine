pub mod collider;
pub mod shape;

use crate::ColliderId;
use nalgebra_glm::{max2, min2, Vec3};

/// Surface-area heuristic broad-phase AABB tree.
///
/// Leaves hold one collider each. Internal nodes hold the union AABB of their
/// subtree. Insertion uses a branch-and-bound DFS to minimise total internal
/// surface area.
pub struct DynamicAABBTree {
    nodes: Vec<Node>,
    node_count: usize,
    root_index: NodeId,
}

impl DynamicAABBTree {
    /// Creates an empty tree. `initial_capacity` pre-allocates node storage to
    /// avoid reallocations if you know the expected number of leaves.
    pub fn new(initial_capacity: usize) -> Self {
        Self {
            nodes: Vec::with_capacity(initial_capacity),
            node_count: 0,
            root_index: 0,
        }
    }

    pub fn insert_leaf(&mut self, aabb: AABB, collider_id: ColliderId) {
        // Step 1: push new leaf, record its id
        let leaf_id = self.nodes.len();
        self.nodes.push(Node {
            aabb,
            collider: Some(collider_id),
            parent: None,
            left: None,
            right: None,
            is_leaf: true,
        });
        self.node_count += 1;

        // Case A: first node becomes root
        if self.node_count == 1 {
            self.root_index = leaf_id;
            return;
        }

        // Step 3: find best sibling
        let sibling_id = self.pick_best(&aabb);

        // Step 4: read sibling's old parent before any push (avoids borrow after realloc)
        let sibling_old_parent = self.nodes[sibling_id].parent;

        // Step 5-6: create new internal node
        let new_parent_aabb = aabb.union(&self.nodes[sibling_id].aabb);
        let new_parent_id = self.nodes.len();
        self.nodes.push(Node {
            aabb: new_parent_aabb,
            collider: None,
            parent: sibling_old_parent,
            left: Some(sibling_id),
            right: Some(leaf_id),
            is_leaf: false,
        });

        // Step 7-8: wire children to new parent
        self.nodes[sibling_id].parent = Some(new_parent_id);
        self.nodes[leaf_id].parent = Some(new_parent_id);

        // Step 9: attach new parent into the tree
        match sibling_old_parent {
            None => {
                // sibling was root
                self.root_index = new_parent_id;
            }
            Some(grandparent_id) => {
                // Copy grandparent to avoid simultaneous borrow
                let gp = self.nodes[grandparent_id];
                if gp.left == Some(sibling_id) {
                    self.nodes[grandparent_id].left = Some(new_parent_id);
                } else {
                    self.nodes[grandparent_id].right = Some(new_parent_id);
                }
                self.refit_ancestors(grandparent_id);
            }
        }
    }

    /// Branch-and-bound DFS sibling selection (minimises sum of internal SA).
    fn pick_best(&self, leaf_aabb: &AABB) -> NodeId {
        let root = self.root_index;
        let root_aabb = self.nodes[root].aabb;

        let mut best_cost = leaf_aabb.union(&root_aabb).area();
        let mut best_node = root;

        // inherited cost for root's children = SA(union(L, root)) - SA(root)
        let root_inherited = best_cost - root_aabb.area();

        // stack: (candidate_id, inherited_cost)
        let mut stack: Vec<(NodeId, f32)> = Vec::new();
        if let Some(l) = self.nodes[root].left {
            stack.push((l, root_inherited));
        }
        if let Some(r) = self.nodes[root].right {
            stack.push((r, root_inherited));
        }

        while let Some((cand_id, inherited)) = stack.pop() {
            let cand_aabb = self.nodes[cand_id].aabb;
            let direct_cost = leaf_aabb.union(&cand_aabb).area();
            let total_cost = direct_cost + inherited;

            if total_cost < best_cost {
                best_cost = total_cost;
                best_node = cand_id;
            }

            // DeltaSA = SA(union(L, cand)) - SA(cand)
            let child_inherited = inherited + direct_cost - cand_aabb.area();

            // Prune: lower bound for any descendant is SA(L) + child_inherited
            if leaf_aabb.area() + child_inherited < best_cost {
                if let Some(l) = self.nodes[cand_id].left {
                    stack.push((l, child_inherited));
                }
                if let Some(r) = self.nodes[cand_id].right {
                    stack.push((r, child_inherited));
                }
            }
        }

        best_node
    }

    /// Walk from `node_id` to root, recomputing each internal node's AABB.
    fn refit_ancestors(&mut self, mut node_id: NodeId) {
        loop {
            // Copy node to read children without holding a borrow
            let node = self.nodes[node_id];
            if let (Some(l), Some(r)) = (node.left, node.right) {
                let merged = self.nodes[l].aabb.union(&self.nodes[r].aabb);
                self.nodes[node_id].aabb = merged;
            }
            match node.parent {
                Some(p) => node_id = p,
                None => break,
            }
        }
    }

    /// Returns the sum of surface areas of all internal nodes. Lower is better.
    /// Useful for comparing tree quality after construction.
    #[allow(dead_code)]
    pub fn compute_cost(&self) -> f32 {
        let mut cost = 0.0;
        for node in &self.nodes {
            if !node.is_leaf {
                cost += node.aabb.area();
            }
        }
        cost
    }

    pub fn clear(&mut self) {
        self.nodes.clear();
        self.node_count = 0;
        self.root_index = 0;
    }

    /// Returns an iterator over the AABBs of all nodes, including internal nodes.
    /// Internal node AABBs are the union of their subtree and are useful for
    /// visualising tree structure.
    pub fn aabbs(&self) -> impl Iterator<Item = &AABB> {
        self.nodes.iter().map(|node| &node.aabb)
    }
}

impl Default for DynamicAABBTree {
    fn default() -> Self {
        Self::new(0)
    }
}

/// Index into the internal node storage of a `DynamicAABBTree`. Internal only.
pub(crate) type NodeId = usize;

// DeltaSA(Node) = SA(node union L) - SA(node)

#[derive(Clone, Copy)]
pub(crate) struct Node {
    aabb: AABB,
    /// Some for leaf nodes, None for internal nodes.
    #[allow(dead_code)]
    collider: Option<ColliderId>,
    parent: Option<NodeId>,
    left: Option<NodeId>,
    right: Option<NodeId>,
    is_leaf: bool,
}

/// Axis-aligned bounding box defined by a lower and upper corner in world space.
#[derive(Clone, Copy, Debug)]
pub struct AABB {
    pub lower: Vec3,
    pub upper: Vec3,
}

impl AABB {
    /// Creates an AABB from explicit lower and upper corners.
    pub fn new(lower: Vec3, upper: Vec3) -> Self {
        Self { lower, upper }
    }

    /// Returns the smallest AABB that contains both `self` and `other`.
    pub fn union(&self, other: &AABB) -> Self {
        let lower = min2(&self.lower, &other.lower);
        let upper = max2(&self.upper, &other.upper);
        Self { lower, upper }
    }

    /// Returns the surface area of the box. Used as the SAH cost metric.
    pub fn area(&self) -> f32 {
        let d = self.upper - self.lower;
        2.0 * (d.x * d.y + d.y * d.z + d.z * d.x)
    }
}
