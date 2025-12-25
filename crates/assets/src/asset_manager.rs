use common::AssetId;
use nalgebra::{Vector2, Vector3};
use rendering_backend::backend_impl::resource_manager::Mesh;
use rendering_backend::vertex::Vertex;
use std::collections::HashMap;
use std::path::Path;
use std::rc::Rc;
use tobj::LoadError;

// Marker types
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ImageAsset {
    pub image_data: Rc<Vec<u8>>,
    pub width: u32,
    pub height: u32,
}

pub struct MeshAsset {
    pub mesh: Rc<Mesh>,
}

pub struct MaterialAsset;

// Convenient aliases

pub type MeshHandle = AssetId<MeshAsset>;
pub type MaterialHandle = AssetId<MaterialAsset>;
pub type ImageHandle = AssetId<ImageAsset>;

pub union Param {
    constant: u32,
    handle: MeshHandle,
}

pub struct AssetManager {
    path_to_mesh_id: HashMap<String, MeshHandle>,
    path_to_image_id: HashMap<String, ImageHandle>,

    id_to_image: HashMap<ImageHandle, Rc<Asset<ImageAsset>>>,
    id_to_mesh: HashMap<MeshHandle, Rc<Asset<MeshAsset>>>,
    next_id: u64,
}

impl AssetManager {
    pub fn get_mesh<P: AsRef<Path>>(&mut self, path: P) -> Option<MeshHandle> {
        let path_str = path.as_ref().to_string_lossy().into_owned(); // Always safe & owned

        if let Some(mesh_asset) = self.path_to_mesh_id.get(&path_str) {
            return Some(*mesh_asset);
        }

        match load_model(path) {
            Ok(mesh_asset) => {
                let asset_id = AssetId::new(self.next_id);
                let mesh_asset_rc = Rc::new(Asset {
                    data: mesh_asset,
                    id: asset_id,
                });
                self.path_to_mesh_id.insert(path_str, asset_id);
                self.id_to_mesh.insert(asset_id, Rc::clone(&mesh_asset_rc));
                self.next_id += 1;

                Some(asset_id)
            }
            Err(e) => {
                eprintln!("AssetManager: Failed to load image {}", e);
                None
            }
        }
    }

    pub fn get_mesh_by_handle(&self, mesh_handle: &MeshHandle) -> Option<Rc<Asset<MeshAsset>>> {
        self.id_to_mesh.get(mesh_handle).map(|x| Rc::clone(x))
    }

    pub fn get_image_by_handle(&self, image_handle: &ImageHandle) -> Option<Rc<Asset<ImageAsset>>> {
        self.id_to_image.get(image_handle).map(|x| Rc::clone(x))
    }

    pub fn get_image<P: AsRef<Path>>(&mut self, path: P) -> Option<ImageHandle> {
        let path_str = path.as_ref().to_str()?.to_string();

        if let Some(asset_id) = self.path_to_image_id.get(&path_str) {
            return Some(*asset_id);
        }

        match load_image(path) {
            Ok(image_asset) => {
                let asset_id = ImageHandle::new(self.next_id);
                let image_asset_rc = Rc::new(Asset {
                    data: image_asset,
                    id: asset_id,
                });
                self.path_to_image_id.insert(path_str, asset_id);
                self.id_to_image
                    .insert(asset_id, Rc::clone(&image_asset_rc));
                self.next_id += 1;
                Some(asset_id)
            }
            Err(e) => {
                eprintln!("AssetManager: Failed to load image {}", e);
                None
            }
        }
    }

    pub fn get_meshes(&self) -> HashMap<u64, Rc<Mesh>> {
        self.id_to_mesh
            .iter()
            .map(|(id, mesh)| (id.id, mesh.data.mesh.clone()))
            .collect()
    }
}

impl Default for AssetManager {
    fn default() -> Self {
        Self {
            path_to_mesh_id: HashMap::new(),
            path_to_image_id: HashMap::new(),
            id_to_image: HashMap::new(),
            id_to_mesh: HashMap::new(),
            next_id: 0,
        }
    }
}

pub struct Asset<T> {
    pub data: T,
    pub id: AssetId<T>,
}

fn load_model<P>(path: P) -> Result<MeshAsset, LoadError>
where
    P: AsRef<Path>,
{
    println!("Loading Model");

    let (models, mat) =
        tobj::load_obj(path.as_ref(), &tobj::GPU_LOAD_OPTIONS).expect("Failed to load model");

    println!("Finished loading Model");
    let model = models.first().expect("No model found");
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
            normal,
            ..Default::default()
        };

        vertices.push(vert);
    }
    println!("Returning");

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
