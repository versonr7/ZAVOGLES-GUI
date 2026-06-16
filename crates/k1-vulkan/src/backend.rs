//! Vulkan backend initialization and management

use ash::{vk, Entry, Instance, Device};
use k1_core::{K1Error, K1Result, Color};
use crate::RendererConfig;

pub struct VulkanBackend {
    #[allow(dead_code)]
    entry: Entry,
    pub instance: Instance,
    pub device: Device,
    pub physical_device: vk::PhysicalDevice,
    pub graphics_queue: vk::Queue,
    pub graphics_queue_family: u32,
    pub present_queue: vk::Queue,
    
    surface: vk::SurfaceKHR,
    surface_loader: ash::khr::surface::Instance,
    
    swapchain: vk::SwapchainKHR,
    swapchain_loader: ash::khr::swapchain::Device,
    swapchain_images: Vec<vk::Image>,
    swapchain_views: Vec<vk::ImageView>,
    swapchain_format: vk::Format,
    swapchain_extent: vk::Extent2D,
    
    render_pass: vk::RenderPass,
    framebuffers: Vec<vk::Framebuffer>,
    
    pipeline_layout: vk::PipelineLayout,
    pipeline: vk::Pipeline,
    
    command_pool: vk::CommandPool,
}

impl VulkanBackend {
    pub fn new(
        native_window: *mut std::ffi::c_void,
        config: &RendererConfig,
    ) -> K1Result<Self> {
        log::info!("K1-Vulkan: Creating Vulkan backend");
        
        // Load Vulkan entry point
        let entry = unsafe { Entry::load() }
            .map_err(|_| K1Error::NotSupported)?;
        
        // Create instance
        let instance = Self::create_instance(&entry, config)?;
        
        // Create surface (Android-specific)
        let (surface, surface_loader) = Self::create_surface(&entry, &instance, native_window)?;
        
        // Select physical device
        let (physical_device, graphics_queue_family, present_queue_family) = 
            Self::select_physical_device(&instance, &surface_loader, surface)?;
        
        // Create logical device
        let (device, graphics_queue, present_queue) = 
            Self::create_device(&instance, physical_device, graphics_queue_family, present_queue_family)?;
        
        // Create swapchain
        let (swapchain, swapchain_loader, swapchain_images, swapchain_format, swapchain_extent) = 
            Self::create_swapchain(&instance, &device, physical_device, &surface_loader, surface, graphics_queue_family, present_queue_family, config)?;
        
        // Create image views
        let swapchain_views = Self::create_image_views(&device, &swapchain_images, swapchain_format)?;
        
        // Create render pass
        let render_pass = Self::create_render_pass(&device, swapchain_format)?;
        
        // Create framebuffers
        let framebuffers = Self::create_framebuffers(&device, &swapchain_views, render_pass, swapchain_extent)?;
        
        // Create pipeline
        let (pipeline_layout, pipeline) = Self::create_pipeline(&device, render_pass, swapchain_extent)?;
        
        // Create command pool
        let command_pool = Self::create_command_pool(&device, graphics_queue_family)?;
        
        Ok(Self {
            entry,
            instance,
            device,
            physical_device,
            graphics_queue,
            graphics_queue_family,
            present_queue,
            surface,
            surface_loader,
            swapchain,
            swapchain_loader,
            swapchain_images,
            swapchain_views,
            swapchain_format,
            swapchain_extent,
            render_pass,
            framebuffers,
            pipeline_layout,
            pipeline,
            command_pool,
        })
    }
    
    fn create_instance(entry: &Entry, config: &RendererConfig) -> K1Result<Instance> {
        let app_info = vk::ApplicationInfo::default()
            .api_version(vk::API_VERSION_1_1);
        
        let layer_names = if config.enable_validation {
            vec![b"VK_LAYER_KHRONOS_validation\0".as_ptr() as *const i8]
        } else {
            vec![]
        };
        
        let extension_names = vec![
            ash::khr::surface::NAME.as_ptr(),
            ash::khr::android_surface::NAME.as_ptr(),
        ];
        
        let create_info = vk::InstanceCreateInfo::default()
            .application_info(&app_info)
            .enabled_layer_names(&layer_names)
            .enabled_extension_names(&extension_names);
        
        unsafe { entry.create_instance(&create_info, None) }
            .map_err(|_| K1Error::NotSupported)
    }
    
    fn create_surface(
        entry: &Entry,
        instance: &Instance,
        native_window: *mut std::ffi::c_void,
    ) -> K1Result<(vk::SurfaceKHR, ash::khr::surface::Instance)> {
        let surface_loader = ash::khr::surface::Instance::new(entry, instance);
        
        let android_surface_loader = ash::khr::android_surface::Instance::new(entry, instance);
        let create_info = vk::AndroidSurfaceCreateInfoKHR::default()
            .window(native_window);
        
        let surface = unsafe {
            android_surface_loader.create_android_surface(&create_info, None)
        }.map_err(|_| K1Error::SurfaceLost)?;
        
        Ok((surface, surface_loader))
    }
    
    fn select_physical_device(
        instance: &Instance,
        surface_loader: &ash::khr::surface::Instance,
        surface: vk::SurfaceKHR,
    ) -> K1Result<(vk::PhysicalDevice, u32, u32)> {
        let devices = unsafe { instance.enumerate_physical_devices() }
            .map_err(|_| K1Error::NotSupported)?;
        
        for &device in &devices {
            let props = unsafe { instance.get_physical_device_properties(device) };
            log::info!("K1-Vulkan: Found device: {:?}", unsafe {
                std::ffi::CStr::from_ptr(props.device_name.as_ptr())
            });
            
            let queue_families = unsafe {
                instance.get_physical_device_queue_family_properties(device)
            };
            
            let mut graphics_family = None;
            let mut present_family = None;
            
            for (i, family) in queue_families.iter().enumerate() {
                if family.queue_flags.contains(vk::QueueFlags::GRAPHICS) {
                    graphics_family = Some(i as u32);
                }
                
                let present_support = unsafe {
                    surface_loader.get_physical_device_surface_support(device, i as u32, surface)
                }.unwrap_or(false);
                
                if present_support {
                    present_family = Some(i as u32);
                }
            }
            
            if let (Some(g), Some(p)) = (graphics_family, present_family) {
                return Ok((device, g, p));
            }
        }
        
        Err(K1Error::NotSupported)
    }
    
    fn create_device(
        instance: &Instance,
        physical_device: vk::PhysicalDevice,
        graphics_queue_family: u32,
        present_queue_family: u32,
    ) -> K1Result<(Device, vk::Queue, vk::Queue)> {
        let queue_priorities = [1.0f32];
        
        let queue_create_infos = if graphics_queue_family == present_queue_family {
            vec![vk::DeviceQueueCreateInfo::default()
                .queue_family_index(graphics_queue_family)
                .queue_priorities(&queue_priorities)]
        } else {
            vec![
                vk::DeviceQueueCreateInfo::default()
                    .queue_family_index(graphics_queue_family)
                    .queue_priorities(&queue_priorities),
                vk::DeviceQueueCreateInfo::default()
                    .queue_family_index(present_queue_family)
                    .queue_priorities(&queue_priorities),
            ]
        };
        
        let device_extensions = vec![
            ash::khr::swapchain::NAME.as_ptr(),
        ];
        
        let features = vk::PhysicalDeviceFeatures::default();
        
        let create_info = vk::DeviceCreateInfo::default()
            .queue_create_infos(&queue_create_infos)
            .enabled_extension_names(&device_extensions)
            .enabled_features(&features);
        
        let device = unsafe {
            instance.create_device(physical_device, &create_info, None)
        }.map_err(|_| K1Error::NotSupported)?;
        
        let graphics_queue = unsafe { device.get_device_queue(graphics_queue_family, 0) };
        let present_queue = unsafe { device.get_device_queue(present_queue_family, 0) };
        
        Ok((device, graphics_queue, present_queue))
    }
    
    fn create_swapchain(
        instance: &Instance,
        device: &Device,
        physical_device: vk::PhysicalDevice,
        surface_loader: &ash::khr::surface::Instance,
        surface: vk::SurfaceKHR,
        graphics_queue_family: u32,
        present_queue_family: u32,
        config: &RendererConfig,
    ) -> K1Result<(vk::SwapchainKHR, ash::khr::swapchain::Device, Vec<vk::Image>, vk::Format, vk::Extent2D)> {
        let surface_caps = unsafe {
            surface_loader.get_physical_device_surface_capabilities(physical_device, surface)
        }.map_err(|_| K1Error::NotSupported)?;
        
        let formats = unsafe {
            surface_loader.get_physical_device_surface_formats(physical_device, surface)
        }.map_err(|_| K1Error::NotSupported)?;
        
        let present_modes = unsafe {
            surface_loader.get_physical_device_surface_present_modes(physical_device, surface)
        }.map_err(|_| K1Error::NotSupported)?;
        
        let surface_format = formats.first().copied()
            .unwrap_or(vk::SurfaceFormatKHR {
                format: vk::Format::B8G8R8A8_UNORM,
                color_space: vk::ColorSpaceKHR::SRGB_NONLINEAR,
            });
        
        let present_mode = if config.vsync {
            vk::PresentModeKHR::FIFO
        } else {
            present_modes.iter()
                .find(|&&m| m == vk::PresentModeKHR::MAILBOX)
                .copied()
                .unwrap_or(vk::PresentModeKHR::FIFO)
        };
        
        let extent = if surface_caps.current_extent.width != u32::MAX {
            surface_caps.current_extent
        } else {
            vk::Extent2D {
                width: config.width.clamp(
                    surface_caps.min_image_extent.width,
                    surface_caps.max_image_extent.width,
                ),
                height: config.height.clamp(
                    surface_caps.min_image_extent.height,
                    surface_caps.max_image_extent.height,
                ),
            }
        };
        
        let image_count = surface_caps.min_image_count + 1;
        let image_count = image_count.min(surface_caps.max_image_count);
        
        let mut create_info = vk::SwapchainCreateInfoKHR::default()
            .surface(surface)
            .min_image_count(image_count)
            .image_format(surface_format.format)
            .image_color_space(surface_format.color_space)
            .image_extent(extent)
            .image_array_layers(1)
            .image_usage(vk::ImageUsageFlags::COLOR_ATTACHMENT)
            .pre_transform(surface_caps.current_transform)
            .composite_alpha(vk::CompositeAlphaFlagsKHR::OPAQUE)
            .present_mode(present_mode)
            .clipped(true);
        
        let queue_family_indices = [graphics_queue_family, present_queue_family];
        if graphics_queue_family != present_queue_family {
            create_info = create_info
                .image_sharing_mode(vk::SharingMode::CONCURRENT)
                .queue_family_indices(&queue_family_indices);
        } else {
            create_info = create_info.image_sharing_mode(vk::SharingMode::EXCLUSIVE);
        }
        
        let swapchain_loader = ash::khr::swapchain::Device::new(instance, device);
        let swapchain = unsafe {
            swapchain_loader.create_swapchain(&create_info, None)
        }.map_err(|_| K1Error::NotSupported)?;
        
        let swapchain_images = unsafe {
            swapchain_loader.get_swapchain_images(swapchain)
        }.map_err(|_| K1Error::NotSupported)?;
        
        Ok((swapchain, swapchain_loader, swapchain_images, surface_format.format, extent))
    }
    
    fn create_image_views(
        device: &Device,
        images: &[vk::Image],
        format: vk::Format,
    ) -> K1Result<Vec<vk::ImageView>> {
        images.iter().map(|&image| {
            let create_info = vk::ImageViewCreateInfo::default()
                .image(image)
                .view_type(vk::ImageViewType::TYPE_2D)
                .format(format)
                .subresource_range(vk::ImageSubresourceRange {
                    aspect_mask: vk::ImageAspectFlags::COLOR,
                    base_mip_level: 0,
                    level_count: 1,
                    base_array_layer: 0,
                    layer_count: 1,
                });
            
            unsafe { device.create_image_view(&create_info, None) }
                .map_err(|_| K1Error::NotSupported)
        }).collect()
    }
    
    fn create_render_pass(device: &Device, format: vk::Format) -> K1Result<vk::RenderPass> {
        let color_attachment = vk::AttachmentDescription::default()
            .format(format)
            .samples(vk::SampleCountFlags::TYPE_1)
            .load_op(vk::AttachmentLoadOp::CLEAR)
            .store_op(vk::AttachmentStoreOp::STORE)
            .stencil_load_op(vk::AttachmentLoadOp::DONT_CARE)
            .stencil_store_op(vk::AttachmentStoreOp::DONT_CARE)
            .initial_layout(vk::ImageLayout::UNDEFINED)
            .final_layout(vk::ImageLayout::PRESENT_SRC_KHR);
        
        let color_attachment_ref = vk::AttachmentReference::default()
            .attachment(0)
            .layout(vk::ImageLayout::COLOR_ATTACHMENT_OPTIMAL);
        
        let subpass = vk::SubpassDescription::default()
            .pipeline_bind_point(vk::PipelineBindPoint::GRAPHICS)
            .color_attachments(std::slice::from_ref(&color_attachment_ref));
        
        let dependency = vk::SubpassDependency::default()
            .src_subpass(vk::SUBPASS_EXTERNAL)
            .dst_subpass(0)
            .src_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
            .src_access_mask(vk::AccessFlags::empty())
            .dst_stage_mask(vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT)
            .dst_access_mask(vk::AccessFlags::COLOR_ATTACHMENT_WRITE);
        
        let create_info = vk::RenderPassCreateInfo::default()
            .attachments(std::slice::from_ref(&color_attachment))
            .subpasses(std::slice::from_ref(&subpass))
            .dependencies(std::slice::from_ref(&dependency));
        
        unsafe { device.create_render_pass(&create_info, None) }
            .map_err(|_| K1Error::NotSupported)
    }
    
    fn create_framebuffers(
        device: &Device,
        views: &[vk::ImageView],
        render_pass: vk::RenderPass,
        extent: vk::Extent2D,
    ) -> K1Result<Vec<vk::Framebuffer>> {
        views.iter().map(|&view| {
            let create_info = vk::FramebufferCreateInfo::default()
                .render_pass(render_pass)
                .attachments(std::slice::from_ref(&view))
                .width(extent.width)
                .height(extent.height)
                .layers(1);
            
            unsafe { device.create_framebuffer(&create_info, None) }
                .map_err(|_| K1Error::NotSupported)
        }).collect()
    }
    
    fn create_pipeline(
        device: &Device,
        render_pass: vk::RenderPass,
        extent: vk::Extent2D,
    ) -> K1Result<(vk::PipelineLayout, vk::Pipeline)> {
        // Simplified - would load SPIR-V shaders in real implementation
        let layout_info = vk::PipelineLayoutCreateInfo::default();
        let layout = unsafe {
            device.create_pipeline_layout(&layout_info, None)
        }.map_err(|_| K1Error::NotSupported)?;
        
        // Placeholder - real implementation would create graphics pipeline with shaders
        let pipeline = vk::Pipeline::null();
        
        Ok((layout, pipeline))
    }
    
    fn create_command_pool(device: &Device, queue_family: u32) -> K1Result<vk::CommandPool> {
        let create_info = vk::CommandPoolCreateInfo::default()
            .queue_family_index(queue_family)
            .flags(vk::CommandPoolCreateFlags::RESET_COMMAND_BUFFER);
        
        unsafe { device.create_command_pool(&create_info, None) }
            .map_err(|_| K1Error::NotSupported)
    }
    
    pub fn begin_render_pass(&self, command_buffer: vk::CommandBuffer) {
        let clear_color = vk::ClearValue {
            color: vk::ClearColorValue { float32: [0.0, 0.0, 0.0, 1.0] },
        };
        
        let render_pass_info = vk::RenderPassBeginInfo::default()
            .render_pass(self.render_pass)
            .framebuffer(self.framebuffers[0]) // Simplified
            .render_area(vk::Rect2D {
                offset: vk::Offset2D { x: 0, y: 0 },
                extent: self.swapchain_extent,
            })
            .clear_values(std::slice::from_ref(&clear_color));
        
        unsafe {
            self.device.cmd_begin_render_pass(
                command_buffer,
                &render_pass_info,
                vk::SubpassContents::INLINE,
            );
        }
    }
    
    pub fn present(&self, render_finished: vk::Semaphore) -> K1Result<()> {
        // Simplified - would acquire next image and present
        Ok(())
    }
}

impl Drop for VulkanBackend {
    fn drop(&mut self) {
        unsafe {
            self.device.device_wait_idle().ok();
            
            self.device.destroy_command_pool(self.command_pool, None);
            
            for &fb in &self.framebuffers {
                self.device.destroy_framebuffer(fb, None);
            }
            
            self.device.destroy_pipeline(self.pipeline, None);
            self.device.destroy_pipeline_layout(self.pipeline_layout, None);
            self.device.destroy_render_pass(self.render_pass, None);
            
            for &view in &self.swapchain_views {
                self.device.destroy_image_view(view, None);
            }
            
            self.swapchain_loader.destroy_swapchain(self.swapchain, None);
            self.surface_loader.destroy_surface(self.surface, None);
            self.device.destroy_device(None);
            self.instance.destroy_instance(None);
        }
    }
}
