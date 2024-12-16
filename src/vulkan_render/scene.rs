use std::path::Path;
use cgmath::{Matrix4, SquareMatrix, Vector2, Vector3};
use crate::vulkan_render::structs::Vertex;

struct Transform {
    position: Vector3<f32>,
    rotation: Vector3<f32>,
    scale: Vector3<f32>,

    model: Matrix4<f32>,
}

impl Default for Transform {
    fn default() -> Transform {
        Transform {
            position: Vector3::new(0.0,0.0,0.0),
            rotation: Vector3::new(0.0,0.0,0.0),
            scale: Vector3::new(1.0,1.0,1.0),
            model: Matrix4::from_value(1.0)
        }
    }
}

pub struct Entity {
    pub transform: Transform,
    pub mesh: Mesh,

    pub parent: Option<Box<Entity>>,
    pub children: Vec<Entity>,
}

impl Entity {
    pub fn new<P>(path: P)-> Entity where
        P: AsRef<Path>, {
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

        Entity {
            transform: Transform::default(),
            mesh: Mesh {
                vertices,
                indices: mesh.indices.clone(),
            },
            parent: None,
            children: vec![],
        }
    }
}

pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
}