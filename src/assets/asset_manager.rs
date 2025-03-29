use crate::vulkan_render::render_objects::draw_objects::{Mesh, Vertex};
use nalgebra::{Vector2, Vector3};
use std::collections::HashMap;
use std::path::Path;
use std::rc::Rc;

pub struct AssetManager {
    image_assets: HashMap<String, Rc<Asset<ImageAsset>>>,
    mesh_assets: HashMap<String, Rc<Asset<MeshAsset>>>,
}

impl AssetManager {
    pub fn get_mesh<P: AsRef<Path>>(&mut self, path: P) -> Option<Rc<Asset<MeshAsset>>> {
        let path_sting = path.as_ref().to_str().unwrap();
        match self.mesh_assets.get(path_sting) {
            Some(mesh_asset) => Some(Rc::clone(mesh_asset)),
            None => {
                let mesh = load_model(path_sting);
                let mesh_asset = Rc::new(Asset {
                    data: MeshAsset { mesh },
                });

                self.mesh_assets
                    .insert(path_sting.parse().unwrap(), mesh_asset);

                self.get_mesh(path)
            }
        }
    }
}

impl Default for AssetManager {
    fn default() -> Self {
        Self {
            mesh_assets: HashMap::new(),
            image_assets: HashMap::new(),
        }
    }
}

pub struct Asset<T> {
    pub data: T,
}

pub struct ImageAsset {
    pub image_data: Vec<u8>,
    pub width: u32,
    pub height: u32,
}

pub struct MeshAsset {
    pub mesh: Mesh,
}

fn load_model<P>(path: P) -> Mesh
where
    P: AsRef<Path>,
{
    let (models, _mat) =
        tobj::load_obj(path.as_ref(), &tobj::GPU_LOAD_OPTIONS).expect("failed to load model file");

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

        let normal = Vector3::new(
            mesh.normals[i * 3],
            mesh.normals[i * 3 + 1],
            mesh.normals[i * 3 + 2],
        );

        let tex_coord: Vector2<f32> =
            Vector2::new(mesh.texcoords[i * 2], mesh.texcoords[i * 2 + 1]);

        //let normal = mesh.normals[i * 3];

        let vert = Vertex {
            pos,
            color: Vector3::new(1.0, 1.0, 1.0),
            tex_coord,
            normal: normal,
            ..Default::default()
        };

        vertices.push(vert);
    }

    Mesh {
        vertices,
        indices: mesh.indices.clone(),
    }
}
