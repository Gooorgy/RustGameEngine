use assets::AssetStore;
use material::material_manager::{MaterialHandle, MaterialVariant};
use material::{MaterialParameterBinding, MaterialParameterBindingData};
use rendering_backend::backend_impl::resource_manager::ResourceManager;
use rendering_backend::backend_impl::vulkan_backend::VulkanBackend;
use rendering_backend::descriptor::{
    DescriptorBinding, DescriptorLayoutDesc, DescriptorLayoutHandle, DescriptorSetHandle,
    DescriptorType, DescriptorValue, DescriptorWriteDesc, SampledImageInfo, ShaderStage,
};
use rendering_backend::sampler::SamplerHandle;
use std::collections::HashMap;

/// Maps material handles to their GPU descriptor sets and layouts.
/// Allocates GPU resources exactly once per material and caches them for subsequent frames.
pub struct MaterialGpuCache {
    layout_cache: HashMap<MaterialVariant, DescriptorLayoutHandle>,
    descriptor_cache: HashMap<MaterialHandle, DescriptorSetHandle>,
}

impl MaterialGpuCache {
    pub fn new() -> Self {
        Self {
            layout_cache: HashMap::new(),
            descriptor_cache: HashMap::new(),
        }
    }

    /// Returns `(descriptor_set, layout)` for the given material.
    /// GPU resources are allocated and written only on the first call for each `material_handle`.
    pub fn get_or_create(
        &mut self,
        vulkan_backend: &mut VulkanBackend,
        material_handle: MaterialHandle,
        bindings: Vec<MaterialParameterBinding>,
        variant: &MaterialVariant,
        resource_manager: &mut ResourceManager,
        asset_store: &AssetStore,
        basic_sampler: SamplerHandle,
    ) -> (DescriptorSetHandle, DescriptorLayoutHandle) {
        let layout_handle = self.get_or_create_layout(vulkan_backend, &bindings, variant);

        // Return the cached set without any allocation if this material was seen before.
        if let Some(&set_handle) = self.descriptor_cache.get(&material_handle) {
            return (set_handle, layout_handle);
        }

        // First time: allocate, write, and cache.
        let set_handle = vulkan_backend.allocate_descriptor_set(layout_handle);

        let writes = bindings
            .into_iter()
            .map(|binding| {
                let gpu_image = match binding.data {
                    MaterialParameterBindingData::Texture(image_handle) => {
                        let image_asset = asset_store.get(image_handle).unwrap_or_else(|| {
                            panic!("No asset found for image_handle: {}", image_handle.raw())
                        });
                        resource_manager.get_or_create_image(
                            vulkan_backend,
                            image_handle,
                            image_asset,
                        )
                    }
                    MaterialParameterBindingData::PackedTexture(_) => {
                        unimplemented!("Packed textures not supported yet")
                    }
                };
                DescriptorWriteDesc {
                    binding: binding.index,
                    value: DescriptorValue::SampledImage(SampledImageInfo {
                        image: gpu_image,
                        sampler: basic_sampler,
                    }),
                }
            })
            .collect::<Vec<_>>();

        vulkan_backend.update_descriptor_set(set_handle, &writes);
        self.descriptor_cache.insert(material_handle, set_handle);

        (set_handle, layout_handle)
    }

    fn get_or_create_layout(
        &mut self,
        vulkan_backend: &mut VulkanBackend,
        bindings: &[MaterialParameterBinding],
        variant: &MaterialVariant,
    ) -> DescriptorLayoutHandle {
        if let Some(&handle) = self.layout_cache.get(variant) {
            return handle;
        }

        let descriptor_bindings = bindings
            .iter()
            .map(|b| DescriptorBinding {
                stages: ShaderStage::FRAGMENT,
                descriptor_type: DescriptorType::CombinedImageSampler,
                count: 1,
                binding: b.index as u32,
            })
            .collect::<Vec<_>>();

        let handle = vulkan_backend.create_descriptor_layout(DescriptorLayoutDesc {
            bindings: descriptor_bindings,
        });

        self.layout_cache.insert(variant.clone(), handle);
        handle
    }
}
