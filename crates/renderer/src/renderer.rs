use crate::frame_data::{FrameData, ResolutionSettings};
use crate::passes::geometry_renderer::GeometryRenderer;
use crate::render_data::MeshRenderRequest;
use crate::render_scene::{MaterialData, MeshRenderData, RenderScene};
use assets::AssetManager;
use material::material_manager::{MaterialHandle, MaterialManager};
use material::{MaterialParameterBinding, MaterialParameterBindingData};
use rendering_backend::backend_impl::resource_manager::ResourceManager;
use rendering_backend::backend_impl::vulkan_backend::VulkanBackend;
use rendering_backend::camera::CameraMvpUbo;
use rendering_backend::descriptor::{
    DescriptorBinding, DescriptorLayoutDesc, DescriptorLayoutHandle, DescriptorSetHandle,
    DescriptorType, DescriptorValue, DescriptorWriteDesc, SampledImageInfo, ShaderStage,
};
use std::collections::HashMap;

pub struct Renderer {
    frame_data: FrameData,
    layout_cache: HashMap<LayoutKey, DescriptorLayoutHandle>,
    descriptor_cache: HashMap<MaterialHandle, DescriptorSetHandle>,
    geometry_renderer: GeometryRenderer,
}

impl Renderer {
    pub fn new(
        vulkan_backend: &mut VulkanBackend,
        resolution_settings: ResolutionSettings,
    ) -> Self {
        let frame_data = FrameData::new(vulkan_backend, resolution_settings, 1000);
        let geometry_renderer = GeometryRenderer::new();
        Self {
            frame_data,
            layout_cache: HashMap::new(),
            descriptor_cache: HashMap::new(),
            geometry_renderer,
        }
    }

    pub fn draw_frame(
        &mut self,
        vulkan_backend: &mut VulkanBackend,
        mesh_requests: &[MeshRenderRequest],
        material_manager: &mut MaterialManager,
        asset_manager: &mut AssetManager,
        resource_manager: &mut ResourceManager,
        camera: CameraMvpUbo,
    ) {
        let render_scene = self.create_render_scene(
            vulkan_backend,
            mesh_requests,
            material_manager,
            asset_manager,
            resource_manager,
            camera,
        );
        vulkan_backend.begin_frame();

        self.geometry_renderer
            .draw_frame(vulkan_backend, &render_scene, &self.frame_data);

        vulkan_backend.end_frame(self.frame_data.frame_images.gbuffer_albedo);
    }

    fn create_render_scene(
        &mut self,
        vulkan_backend: &mut VulkanBackend,
        mesh_requests: &[MeshRenderRequest],
        material_manager: &mut MaterialManager,
        asset_manager: &mut AssetManager,
        resource_manager: &mut ResourceManager,
        camera: CameraMvpUbo,
    ) -> RenderScene {
        let mut meshes = vec![];
        let mut model_matrices = vec![];

        for request in mesh_requests {
            let mesh_asset = asset_manager
                .get_mesh_by_handle(&request.mesh_handle)
                .expect(
                    format!("No asset found for mesh_handle: {}", request.mesh_handle.id).as_str(),
                );

            let model_matrix = request.transform.get_model_matrix();
            model_matrices.push(model_matrix);

            let gpu_mesh_data = resource_manager.get_or_create_mesh(
                vulkan_backend,
                mesh_asset.id.id,
                mesh_asset.data.mesh.clone(),
            );

            let material_bindings = material_manager.get_material_data(request.material_handle);
            let shader_variant = material_manager.get_variant(request.material_handle);
            let layout_key = LayoutKey {
                shader_path: shader_variant.name.clone(),
            };

            let (set_handle, layout_handle) = self.get_or_create_descriptors(
                vulkan_backend,
                request.material_handle,
                material_bindings,
                resource_manager,
                asset_manager,
                layout_key,
            );

            let mesh_render_data = MeshRenderData {
                mesh_data: gpu_mesh_data,
                material_data: MaterialData {
                    shader_variant: shader_variant.clone(),
                    descriptor_set_handle: set_handle,
                    descriptor_layout_handle: layout_handle,
                    push_constant_data: material_manager
                        .get_material_push_const_data(request.material_handle),
                },
            };

            meshes.push(mesh_render_data);
        }

        vulkan_backend.update_buffer(
            self.frame_data.model_storage_buffer,
            model_matrices.as_slice(),
        );
        vulkan_backend.update_buffer(self.frame_data.camera_buffer, &[camera]);

        RenderScene {
            meshes,
            camera: self.frame_data.camera_buffer,
        }
    }

    fn get_or_create_descriptors(
        &mut self,
        vulkan_backend: &mut VulkanBackend,
        material_handle: MaterialHandle,
        bindings: Vec<MaterialParameterBinding>,
        resource_manager: &mut ResourceManager,
        asset_manager: &mut AssetManager,
        layout_key: LayoutKey,
    ) -> (DescriptorSetHandle, DescriptorLayoutHandle) {
        let layout_handle = self.get_or_create_layout(vulkan_backend, &*bindings, layout_key);
        let descriptors = self.descriptor_cache.get(&material_handle);

        let descriptor_handle = vulkan_backend.allocate_descriptor_set(layout_handle);

        if let Some(descriptors) = descriptors {
            return (*descriptors, layout_handle);
        }

        let mut descriptor_write_descs = vec![];
        for binding in bindings {
            let image_asset = match binding.data {
                MaterialParameterBindingData::Texture(image_handle) => {
                    asset_manager.get_image_by_handle(&image_handle).expect(
                        format!("No asset found for image_handle: {}", image_handle.id).as_str(),
                    )
                }
                MaterialParameterBindingData::PackedTexture(_) => {
                    unimplemented!("Packed textures not supported yet")
                }
            };

            let gpu_image_handle = resource_manager.get_or_create_image(
                vulkan_backend,
                image_asset.id.id,
                image_asset.data.image_data.clone(),
                image_asset.data.width,
                image_asset.data.height,
            );

            let descriptor_write_desc = DescriptorWriteDesc {
                binding: binding.index,
                value: DescriptorValue::SampledImage(SampledImageInfo {
                    image: gpu_image_handle,
                    sampler: self.frame_data.basic_sampler,
                }),
            };

            descriptor_write_descs.push(descriptor_write_desc);
        }

        vulkan_backend.update_descriptor_set(descriptor_handle, &*descriptor_write_descs);

        (descriptor_handle, layout_handle)
    }

    fn get_or_create_layout(
        &mut self,
        vulkan_backend: &mut VulkanBackend,
        bindings: &[MaterialParameterBinding],
        layout_key: LayoutKey,
    ) -> DescriptorLayoutHandle {
        let layout_handle = self.layout_cache.get(&layout_key);
        if let Some(layout_handle) = layout_handle {
            return *layout_handle;
        }

        let descriptor_bindings = bindings
            .iter()
            .map(|material_binding| DescriptorBinding {
                stages: ShaderStage::FRAGMENT,
                descriptor_type: DescriptorType::CombinedImageSampler,
                count: 1,
                binding: material_binding.index as u32,
            })
            .collect::<Vec<_>>();

        let layout_desc = DescriptorLayoutDesc {
            bindings: descriptor_bindings,
        };

        let layout_handle = vulkan_backend.create_descriptor_layout(layout_desc);
        self.layout_cache.insert(layout_key, layout_handle.clone());

        layout_handle
    }
}

#[derive(Eq, PartialEq, Hash)]
pub struct LayoutKey {
    pub shader_path: String,
}
