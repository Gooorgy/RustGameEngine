use crate::assign_texture;
use assets::{Asset, AssetManager, ImageAsset};
use std::rc::Rc;

pub struct Material {
    pub name: String,
    pub material_type: MaterialType,
}

pub enum MaterialType {
    Pbr(PbrMaterialInstance),
}

pub struct PbrMaterialInstance {
    albedo_texture: Option<String>,
    albedo_texture_asset: Option<Rc<Asset<ImageAsset>>>,
    normal_texture: Option<String>,
    normal_texture_asset: Option<Rc<Asset<ImageAsset>>>,
    metallic_texture: Option<String>,
    metallic_texture_asset: Option<Rc<Asset<ImageAsset>>>,
    roughness_texture: Option<String>,
    roughness_texture_asset: Option<Rc<Asset<ImageAsset>>>,
    emissive_texture: Option<String>,
    emissive_texture_asset: Option<Rc<Asset<ImageAsset>>>,
}

impl MaterialInstance for PbrMaterialInstance {
    fn init(&mut self, asset_manager: &mut AssetManager) {
        assign_texture!(
            &self.albedo_texture,
            asset_manager,
            self.albedo_texture_asset
        );
        assign_texture!(
            &self.normal_texture,
            asset_manager,
            self.normal_texture_asset
        );
        assign_texture!(
            &self.metallic_texture,
            asset_manager,
            self.metallic_texture_asset
        );
        assign_texture!(
            &self.roughness_texture,
            asset_manager,
            self.roughness_texture_asset
        );
        assign_texture!(
            &self.emissive_texture,
            asset_manager,
            self.emissive_texture_asset
        );
    }
}

impl Default for PbrMaterialInstance {
    fn default() -> Self {
        PbrMaterialInstance {
            albedo_texture: None,
            albedo_texture_asset: None,
            normal_texture: None,
            normal_texture_asset: None,
            metallic_texture: None,
            metallic_texture_asset: None,
            roughness_texture: None,
            roughness_texture_asset: None,
            emissive_texture: None,
            emissive_texture_asset: None,
        }
    }
}

impl PbrMaterialInstance {
    pub fn with_albedo_texture(mut self, albedo_texture: &str) -> Self {
        self.albedo_texture = Some(albedo_texture.to_string());
        self
    }

    pub fn with_normal_texture(mut self, normal_texture: &str) -> Self {
        self.albedo_texture = Some(normal_texture.to_string());
        self
    }

    pub fn with_metallic_texture(mut self, metallic_texture: &str) -> Self {
        self.metallic_texture = Some(metallic_texture.to_string());
        self
    }

    pub fn with_emissive_texture(mut self, emissive_texture: &str) -> Self {
        self.emissive_texture = Some(emissive_texture.to_string());
        self
    }

    pub fn with_roughness_texture(mut self, roughness_texture: &str) -> Self {
        self.roughness_texture = Some(roughness_texture.to_string());
        self
    }
}

impl Default for Material {
    fn default() -> Self {
        Material {
            material_type: MaterialType::Pbr(PbrMaterialInstance::default()),
            name: String::from("material"),
        }
    }
}

impl Material {
    pub fn pbr(name: &str, material_instance: PbrMaterialInstance) -> Self {
        Material {
            material_type: MaterialType::Pbr(material_instance),
            name: String::from(name),
        }
    }
}

pub trait MaterialInstance {
    fn init(&mut self, asset_manager: &mut AssetManager);
}
