use nalgebra::{Vector2, Vector3};
use std::collections::HashMap;
use std::path::Path;
use std::rc::Rc;
use tobj::LoadError;
use vulkan_backend::render_objects::draw_objects::{Mesh, Vertex};

pub struct AssetManager {
    image_assets: HashMap<String, Rc<Asset<ImageAsset>>>,
    mesh_assets: HashMap<String, Rc<Asset<MeshAsset>>>,
}

impl AssetManager {
    pub fn get_mesh<P: AsRef<Path>>(&mut self, path: P) -> Option<Rc<Asset<MeshAsset>>> {
        let path_str = path.as_ref().to_str()?.to_string();

        if let Some(mesh_asset) = self.mesh_assets.get(&path_str) {
            return Some(Rc::clone(mesh_asset));
        }

        match load_model(path) {
            Ok(mesh_asset) => {
                let mesh_asset_rc = Rc::new(Asset { data: mesh_asset });
                self.mesh_assets.insert(path_str, Rc::clone(&mesh_asset_rc));
                Some(mesh_asset_rc)
            }
            Err(e) => {
                eprintln!("AssetManager: Failed to load image {}", e);
                None
            }
        }
    }

    pub fn get_image<P: AsRef<Path>>(&mut self, path: P) -> Option<Rc<Asset<ImageAsset>>> {
        let path_str = path.as_ref().to_str()?.to_string();

        if let Some(image_asset) = self.image_assets.get(&path_str) {
            return Some(Rc::clone(image_asset));
        }

        match load_image(path) {
            Ok(image_asset) => {
                let image_asset_rc = Rc::new(Asset { data: image_asset });
                self.image_assets
                    .insert(path_str, Rc::clone(&image_asset_rc));
                Some(image_asset_rc)
            }
            Err(e) => {
                eprintln!("AssetManager: Failed to load image {}", e);
                None
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
    pub image_data: Rc<Vec<u8>>,
    pub width: u32,
    pub height: u32,
}

pub struct MeshAsset {
    pub mesh: Rc<Mesh>,
}

fn load_model<P>(path: P) -> Result<MeshAsset, LoadError>
where
    P: AsRef<Path>,
{
    let (models, _mat) = tobj::load_obj(path.as_ref(), &tobj::GPU_LOAD_OPTIONS)?;

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

        let vert = Vertex {
            pos,
            color: Vector3::new(1.0, 1.0, 1.0),
            tex_coord,
            normal: normal,
            ..Default::default()
        };

        vertices.push(vert);
    }

    Ok(MeshAsset {
        mesh: Rc::new(Mesh {
            vertices,
            indices: mesh.indices.clone(),
        }),
    })
}

pub fn load_image<P>(path: P) -> Result<ImageAsset, image::ImageError>
where
    P: AsRef<Path>,
{
    let dyn_image = image::open(path)?;
    let image_width = dyn_image.width();
    let image_height = dyn_image.height();

    let image_data = dyn_image.to_rgba8().into_raw();

    Ok(ImageAsset {
        image_data: Rc::new(image_data),
        width: image_width,
        height: image_height,
    })
}
