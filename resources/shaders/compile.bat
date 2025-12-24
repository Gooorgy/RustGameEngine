C:\VulkanSDK\1.3.290.0\Bin/glslc.exe shader.vert -o vert.spv
C:\VulkanSDK\1.3.290.0\Bin/glslc.exe shader.frag -o pbr.frag.base.spv
C:\VulkanSDK\1.3.290.0\Bin/glslc.exe shader.frag -DHAS_BASE_COLOR -o pbr.frag.color.spv

C:\VulkanSDK\1.3.290.0\Bin/glslc.exe shader.frag -DHAS_NORMAL -o pbr.frag.normal.spv
C:\VulkanSDK\1.3.290.0\Bin/glslc.exe shader.frag -DHAS_NORMAL -DHAS_BASE_COLOR -o pbr.frag.normal.color.spv

C:\VulkanSDK\1.3.290.0\Bin/glslc.exe shader.frag -DHAS_ORM -o pbr.frag.orm.spv
C:\VulkanSDK\1.3.290.0\Bin/glslc.exe shader.frag -DHAS_ORM -DHAS_BASE_COLOR -o pbr.frag.orm.color.spv

C:\VulkanSDK\1.3.290.0\Bin/glslc.exe shader.frag -DHAS_ORM -DHAS_NORMAL -o pbr.frag.orm.normal.spv
C:\VulkanSDK\1.3.290.0\Bin/glslc.exe shader.frag -DHAS_ORM -DHAS_NORMAL -DHAS_BASE_COLOR -o pbr.frag.orm.normal.color.spv

C:\VulkanSDK\1.3.290.0\Bin/glslc.exe shadow.vert -o shadow.spv
C:\VulkanSDK\1.3.290.0\Bin/glslc.exe lighting.frag -o lighting.spv
C:\VulkanSDK\1.3.290.0\Bin/glslc.exe quad.vert -o quad.spv
C:\VulkanSDK\1.3.290.0\Bin/glslc.exe line_debug.vert -o line_debug_vert.spv
C:\VulkanSDK\1.3.290.0\Bin/glslc.exe line_debug.frag -o line_debug_frag.spv

pause