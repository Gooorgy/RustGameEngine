use crate::emesh::read_emesh;
use common::AssetId;
use rendering_backend::backend_impl::resource_manager::Mesh;
use std::collections::HashMap;
use std::path::Path;
use std::rc::Rc;

// Marker types

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct ImageAsset {
    pub image_data: Rc<Vec<u8>>,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug)]
pub struct MeshAsset {
    pub mesh: Rc<Mesh>,
}

pub struct MaterialAsset;

// Convenient aliases

pub type MeshHandle = AssetId<MeshAsset>;
pub type ImageHandle = AssetId<ImageAsset>;

#[derive(Default)]
pub struct AssetManager {
    path_to_mesh_id: HashMap<String, MeshHandle>,
    path_to_image_id: HashMap<String, ImageHandle>,

    id_to_image: HashMap<ImageHandle, Rc<Asset<ImageAsset>>>,
    id_to_mesh: HashMap<MeshHandle, Rc<Asset<MeshAsset>>>,
    next_id: u64,
}

impl AssetManager {
    /// Loads a cooked `.emesh` file. Returns a cached handle if the same path
    /// was already loaded.
    pub fn load_emesh(&mut self, path: &Path) -> Option<MeshHandle> {
        let path_str = path.to_string_lossy().into_owned();

        if let Some(&handle) = self.path_to_mesh_id.get(&path_str) {
            return Some(handle);
        }

        match read_emesh(path) {
            Ok(mesh) => {
                let asset_id = AssetId::new(self.next_id);
                let asset = Rc::new(Asset { data: MeshAsset { mesh }, id: asset_id });
                self.path_to_mesh_id.insert(path_str, asset_id);
                self.id_to_mesh.insert(asset_id, asset);
                self.next_id += 1;
                Some(asset_id)
            }
            Err(e) => {
                eprintln!("AssetManager: failed to load '{}': {}", path.display(), e);
                None
            }
        }
    }

    pub fn get_mesh_by_handle(&self, mesh_handle: &MeshHandle) -> Option<Rc<Asset<MeshAsset>>> {
        self.id_to_mesh.get(mesh_handle).map(Rc::clone)
    }

    pub fn get_image_by_handle(&self, image_handle: &ImageHandle) -> Option<Rc<Asset<ImageAsset>>> {
        self.id_to_image.get(image_handle).map(Rc::clone)
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
                self.id_to_image.insert(asset_id, Rc::clone(&image_asset_rc));
                self.next_id += 1;
                Some(asset_id)
            }
            Err(e) => {
                eprintln!("AssetManager: failed to load image: {}", e);
                None
            }
        }
    }

    pub fn get_meshes(&self) -> HashMap<u64, Rc<Mesh>> {
        self.id_to_mesh
            .iter()
            .map(|(id, mesh)| (id.raw(), mesh.data.mesh.clone()))
            .collect()
    }
}

pub struct Asset<T> {
    pub data: T,
    pub id: AssetId<T>,
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