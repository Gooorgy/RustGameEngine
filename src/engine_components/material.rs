use glm::Vec3;
use std::path::Path;
use vulkan_backend::scene::ImageResource;

#[derive(Debug, Default)]
pub struct OpaqueMaterial {
    pub albedo_texture: Option<ImageResource>,
    pub normal_texture: Option<ImageResource>,
    pub metallic_texture: Option<ImageResource>,
    pub roughness_texture: Option<ImageResource>,
    pub emissive_texture: Option<ImageResource>,

    pub base_color: Option<Vec3>,
}

impl OpaqueMaterial {
    pub fn with_albedo_texture<P: AsRef<Path>>(&mut self, path: P) {
        let image_resource = ImageResource::load_from_file(path);
        self.albedo_texture = Some(image_resource);
    }

    pub fn with_normal_texture<P: AsRef<Path>>(&mut self, path: P) {
        let image_resource = ImageResource::load_from_file(path);
        self.normal_texture = Some(image_resource);
    }

    pub fn with_metallic_texture<P: AsRef<Path>>(&mut self, path: P) {
        let image_resource = ImageResource::load_from_file(path);
        self.metallic_texture = Some(image_resource);
    }

    pub fn with_roughness_texture<P: AsRef<Path>>(&mut self, path: P) {
        let image_resource = ImageResource::load_from_file(path);
        self.roughness_texture = Some(image_resource);
    }

    pub fn with_base_color(&mut self, base_color: Vec3) {
        self.base_color = Some(base_color);
    }
}
