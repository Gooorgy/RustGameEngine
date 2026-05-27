use crate::frame_data::{FrameData, ResolutionSettings};
use crate::material_gpu_cache::MaterialGpuCache;
use crate::passes::aabb_debug_renderer::AabbDebugRenderer;
use crate::passes::geometry_renderer::GeometryRenderer;
use crate::passes::lighting_renderer::LightingRenderer;
use crate::render_data::{CameraRenderData, DirectionalLightData, MeshRenderRequest};
use crate::render_scene::{MaterialData, MeshRenderData, RenderScene};
use crate::shader_loader::ShaderCache;
use assets::AssetStore;
use common::MeshData;
use material::material_manager::MaterialManager;
use rendering_backend::backend_impl::resource_manager::ResourceManager;
use rendering_backend::backend_impl::vulkan_backend::VulkanBackend;
use rendering_backend::camera::CameraMvpUbo;
use std::path::PathBuf;

pub use crate::passes::aabb_debug_renderer::DebugBox;

pub struct RendererConfig {
    pub resolution_settings: ResolutionSettings,
    /// Directory containing cook-time asset shaders from the project cache.
    pub asset_cache_dir: PathBuf,
}

pub struct Renderer {
    frame_data: FrameData,
    material_gpu_cache: MaterialGpuCache,
    geometry_renderer: GeometryRenderer,
    lighting_renderer: LightingRenderer,
    aabb_debug_renderer: AabbDebugRenderer,
    shader_cache: ShaderCache,
}

impl Renderer {
    pub fn new(vulkan_backend: &mut VulkanBackend, config: RendererConfig) -> Self {
        let frame_data = FrameData::new(vulkan_backend, config.resolution_settings, 1000);
        let geometry_renderer = GeometryRenderer::new();
        let aabb_debug_renderer = AabbDebugRenderer::new(vulkan_backend);
        let mut shader_cache = ShaderCache::new(config.asset_cache_dir);
        let lighting_renderer =
            LightingRenderer::new(vulkan_backend, &frame_data, &mut shader_cache);
        Self {
            frame_data,
            material_gpu_cache: MaterialGpuCache::new(),
            geometry_renderer,
            lighting_renderer,
            aabb_debug_renderer,
            shader_cache,
        }
    }

    pub fn toggle_aabb_debug(&mut self) {
        self.aabb_debug_renderer.toggle();
    }

    #[allow(clippy::too_many_arguments)]
    pub fn draw_frame(
        &mut self,
        vulkan_backend: &mut VulkanBackend,
        mesh_requests: &[MeshRenderRequest],
        material_manager: &mut MaterialManager,
        asset_store: &AssetStore,
        resource_manager: &mut ResourceManager,
        camera: CameraMvpUbo,
        camera_render_data: Option<CameraRenderData>,
        directional_light: Option<DirectionalLightData>,
        aabbs: &[DebugBox],
    ) {
        let render_scene = self.create_render_scene(
            vulkan_backend,
            mesh_requests,
            material_manager,
            asset_store,
            resource_manager,
            camera,
            camera_render_data,
            directional_light,
        );
        vulkan_backend.begin_frame();

        self.geometry_renderer.draw_frame(
            vulkan_backend,
            &render_scene,
            &self.frame_data,
            &mut self.shader_cache,
        );
        self.lighting_renderer.draw_frame(vulkan_backend, &render_scene, &self.frame_data);
        self.aabb_debug_renderer.draw_frame(
            vulkan_backend,
            aabbs,
            &self.frame_data,
            camera,
            &mut self.shader_cache,
        );

        vulkan_backend.end_frame(self.frame_data.frame_images.draw_image);
    }

    #[allow(clippy::too_many_arguments)]
    fn create_render_scene(
        &mut self,
        vulkan_backend: &mut VulkanBackend,
        mesh_requests: &[MeshRenderRequest],
        material_manager: &mut MaterialManager,
        asset_store: &AssetStore,
        resource_manager: &mut ResourceManager,
        camera: CameraMvpUbo,
        camera_render_data: Option<CameraRenderData>,
        directional_light: Option<DirectionalLightData>,
    ) -> RenderScene {
        let mut meshes = vec![];
        let mut model_matrices = vec![];

        let basic_sampler = self.frame_data.basic_sampler;

        for request in mesh_requests {
            let mesh_data = asset_store
                .get::<MeshData>(request.mesh_handle)
                .unwrap_or_else(|| {
                    panic!("No asset found for mesh_handle: {}", request.mesh_handle)
                });

            model_matrices.push(request.transform.get_model_matrix());

            let gpu_mesh_data =
                resource_manager.get_or_create_mesh(vulkan_backend, request.mesh_handle, mesh_data);

            let material_bindings =
                material_manager.get_bindings(request.material_handle).to_vec();
            let shader_variant = material_manager.get_variant(request.material_handle).clone();
            let push_constant_data =
                material_manager.get_push_constants(request.material_handle).to_vec();

            let (set_handle, layout_handle) = self.material_gpu_cache.get_or_create(
                vulkan_backend,
                request.material_handle,
                material_bindings,
                &shader_variant,
                resource_manager,
                asset_store,
                basic_sampler,
            );

            meshes.push(MeshRenderData {
                mesh_data: gpu_mesh_data,
                material_data: MaterialData {
                    shader_variant,
                    descriptor_set_handle: set_handle,
                    descriptor_layout_handle: layout_handle,
                    push_constant_data,
                },
            });
        }

        vulkan_backend.update_buffer(
            self.frame_data.model_storage_buffer,
            model_matrices.as_slice(),
        );
        vulkan_backend.update_buffer(self.frame_data.camera_buffer, &[camera]);

        RenderScene {
            meshes,
            camera_data: camera_render_data,
            directional_light,
        }
    }
}