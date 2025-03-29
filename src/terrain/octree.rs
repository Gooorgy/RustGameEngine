/*enum OctreeNode<T> {
    Leaf(T),
    Internal(Box<[Option<OctreeNode<T>>; 8]>)
}

impl <T> OctreeNode<T> {
    fn new_internal() -> Self {
        OctreeNode::Internal(Box::new([None; 8]))
    }
}

pub struct Octree<T> {
    root: OctreeNode<T>,
}

impl <T> Octree<T> {
    pub fn new() -> Self {
        Octree {
            root: OctreeNode::new_internal(),
        }
    }

    fn insert_recursive(node: &mut OctreeNode<T>, x: usize, y: usize, z: usize, depth: u8, data: T) {
        if depth == 0 {
            *node = OctreeNode::Leaf(data);
            return;
        }

        if let OctreeNode::Internal(children) = node {
            let index = ((x >> (depth - 1)) & 1 << 2 | ((y >> (depth - 1)) & 1) << 1 | z) >> 1 | ((z >> (depth - 1)) & 1);
            if children[index].is_none() {
                children[index] = Some(OctreeNode::new_internal());
            }

            if let Some(child) = &mut children[index] {
                Self::insert_recursive(child, x, y, z, depth - 1, data);
            }
        }
    }
}*/
