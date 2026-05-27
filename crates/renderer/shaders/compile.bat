C:\VulkanSDK\1.3.290.0\Bin\glslc.exe shader.vert -o vert.spv
C:\VulkanSDK\1.3.290.0\Bin\glslc.exe shader.frag -o pbr.frag.spv
C:\VulkanSDK\1.3.290.0\Bin\glslc.exe shader.frag -DHAS_COLOR_TEXTURE -o pbr.frag.HAS_COLOR_TEXTURE.spv
C:\VulkanSDK\1.3.290.0\Bin\glslc.exe shader.frag -DHAS_NORMAL_TEXTURE -o pbr.frag.HAS_NORMAL_TEXTURE.spv
C:\VulkanSDK\1.3.290.0\Bin\glslc.exe shader.frag -DHAS_ORM_TEXTURE -o pbr.frag.HAS_ORM_TEXTURE.spv
C:\VulkanSDK\1.3.290.0\Bin\glslc.exe shader.frag -DHAS_COLOR_TEXTURE -DHAS_NORMAL_TEXTURE -o pbr.frag.HAS_COLOR_TEXTURE.HAS_NORMAL_TEXTURE.spv
C:\VulkanSDK\1.3.290.0\Bin\glslc.exe shader.frag -DHAS_COLOR_TEXTURE -DHAS_ORM_TEXTURE -o pbr.frag.HAS_COLOR_TEXTURE.HAS_ORM_TEXTURE.spv
C:\VulkanSDK\1.3.290.0\Bin\glslc.exe shader.frag -DHAS_NORMAL_TEXTURE -DHAS_ORM_TEXTURE -o pbr.frag.HAS_NORMAL_TEXTURE.HAS_ORM_TEXTURE.spv
C:\VulkanSDK\1.3.290.0\Bin\glslc.exe shader.frag -DHAS_COLOR_TEXTURE -DHAS_NORMAL_TEXTURE -DHAS_ORM_TEXTURE -o pbr.frag.HAS_COLOR_TEXTURE.HAS_NORMAL_TEXTURE.HAS_ORM_TEXTURE.spv

C:\VulkanSDK\1.3.290.0\Bin\glslc.exe shadow.vert -o shadow.spv
C:\VulkanSDK\1.3.290.0\Bin\glslc.exe lighting.frag -o lighting.spv
C:\VulkanSDK\1.3.290.0\Bin\glslc.exe quad.vert -o quad.spv
C:\VulkanSDK\1.3.290.0\Bin\glslc.exe line_debug.vert -o line_debug_vert.spv
C:\VulkanSDK\1.3.290.0\Bin\glslc.exe line_debug.frag -o line_debug_frag.spv

pause