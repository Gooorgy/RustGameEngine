use std::cell::{RefCell, UnsafeCell};
use std::ops::Deref;
use std::path::Path;
use std::rc::{Rc, Weak};
use nalgebra::{Matrix4, Vector2, Vector3};
use typed_arena::Arena;
use crate::vulkan_render::structs::Vertex;

pub struct Transform {
    pub position: Vector3<f32>,
    pub rotation: Vector3<f32>,
    pub scale: Vector3<f32>,

    pub model: Matrix4<f32>,
}

impl Default for Transform {
    fn default() -> Transform {
        Transform {
            position: Vector3::new(0.0,0.0,0.0),
            rotation: Vector3::new(0.0,0.0,0.0),
            scale: Vector3::new(1.0,1.0,1.0),
            model: Matrix4::identity(),
        }
    }
}

pub struct SceneNode {
    pub transform: Transform,
    pub mesh: Mesh,

    pub parent: Option<Weak<RefCell<SceneNode>>>,
    pub children: Vec<Rc<RefCell<SceneNode>>>,
}

impl SceneNode {
    pub fn new<P>(path: P)-> Rc<RefCell<Self>>
        where P: AsRef<Path>, {
        let (models, mat) = tobj::load_obj(
            path.as_ref(),
            &tobj::GPU_LOAD_OPTIONS,
        )
            .expect("failed to load model file");

        let model = models.first().unwrap();
        let mesh = &model.mesh;

        let mut vertices = vec![];

        let vert_count = mesh.positions.len() / 3;
        for i in 0..vert_count {
            let pos: Vector3<f32> = Vector3::new(
                mesh.positions[i * 3],
                mesh.positions[i * 3 + 1],
                mesh.positions[i * 3 + 2],
            );

            let tex_coord: Vector2<f32> = Vector2::new(
                mesh.texcoords[i * 2],
                mesh.texcoords[i * 2 + 1]);

            let vert = Vertex {
                pos,
                color: Vector3::new(1.0, 1.0, 1.0),
                tex_coord,
            };

            vertices.push(vert);
        }
        let mut transform = Transform::default();
        transform.scale = Vector3::new(0.75,0.75,0.75);
        transform.position = Vector3::new(0.0,1.0,0.5);
        Rc::new(RefCell::new(SceneNode {
            transform,
            mesh: Mesh {
                vertices,
                indices: mesh.indices.clone(),
            },
            parent: None,
            children: Vec::new(),
        }))
    }

    pub fn add_child<P>(parent: Rc<RefCell<SceneNode>>, path: P) where P: AsRef<Path> {
        let x = Self::new(path);

        parent.borrow_mut().children.push(x.clone());
        x.borrow_mut().set_parent(Some(parent));
    }

    pub fn remove_child(&mut self, child: &Rc<RefCell<SceneNode>>) {
        if let Some(pos) = self.children.iter().position(|x| Rc::ptr_eq(x, child)) {
            self.children.remove(pos);
        }
    }

    pub fn get_local__model_matrix(&self) -> Matrix4<f32> {
        let transform_x = glm::rotate(&Matrix4::identity(), self.transform.rotation.x.to_radians(), &Vector3::new(1.0,0.0,0.0));
        let transform_y = glm::rotate(&Matrix4::identity(), self.transform.rotation.y.to_radians(), &Vector3::new(0.0,1.0,0.0));
        let transform_z = glm::rotate(&Matrix4::identity(), self.transform.rotation.z.to_radians(), &Vector3::new(0.0,0.0,1.0));

        let rotation_matrix = transform_x * transform_y * transform_z;

        glm::translate(&Matrix4::identity(), &self.transform.position) * rotation_matrix * glm::scale(&Matrix4::identity(), &self.transform.scale)
    }

    pub fn get_transform(&self) -> &Transform {
        &self.transform
    }

    pub fn update(node: Rc<RefCell<SceneNode>>) {
        {
           let mut m_node = node.borrow_mut();

            match m_node.parent.clone() {
                Some(p) => {
                    let parent_model = p.upgrade().unwrap().borrow().transform.model;
                    m_node.transform.model = parent_model * m_node.get_local__model_matrix();
                },
                _ => {
                    m_node.transform.model = m_node.get_local__model_matrix();
                }
            };
        }
        for child in node.borrow().children.iter() {
            Self::update(child.clone());
        }
    }

    fn set_parent(&mut self, parent: Option<Rc<RefCell<SceneNode>>>) {
        self.parent = parent.map(|p| Rc::downgrade(&p));
    }
}


pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
}