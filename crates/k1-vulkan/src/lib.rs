//! K1-Vulkan: Raw Vulkan backend for K1 Platform
//! Zero-allocation, pre-allocated everything

pub mod backend;
pub mod commands;
pub mod resources;
pub mod pipeline;

use k1_core::{K1Error, K1Result, Color, Vertex, Rect};
use ash::vk;

/// Maximum frames in flight (triple buffering)
pub const MAX_FRAMES_IN_FLIGHT: usize = 3;

/// Maximum vertices per frame
pub const MAX_VERTICES_PER_FRAME: usize = 65536;

/// Maximum textures
pub const MAX_TEXTURES: usize = 256;

/// Maximum command buffers
pub const MAX_COMMAND_BUFFERS: usize = 16;

/// Renderer configuration
#[derive(Clone, Debug)]
pub struct RendererConfig {
    pub width: u32,
    pub height: u32,
    pub enable_validation: bool,
    pub vsync: bool,
}

impl RendererConfig {
    pub const fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            enable_validation: false,
            vsync: true,
        }
    }
}

/// Main Vulkan renderer
pub struct VulkanRenderer {
    backend: backend::VulkanBackend,
    frame_data: [FrameData; MAX_FRAMES_IN_FLIGHT],
    current_frame: usize,
}

/// Per-frame data (pre-allocated)
struct FrameData {
    vertex_buffer: vk::Buffer,
    vertex_buffer_memory: vk::DeviceMemory,
    vertex_mapped: *mut Vertex,
    vertex_count: usize,
    
    command_buffer: vk::CommandBuffer,
    command_pool: vk::CommandPool,
    
    image_available: vk::Semaphore,
    render_finished: vk::Semaphore,
    in_flight: vk::Fence,
}

impl VulkanRenderer {
    /// Initialize renderer with native window
    pub fn new(
        native_window: *mut std::ffi::c_void,
        config: RendererConfig,
    ) -> K1Result<Self> {
        log::info!("K1-Vulkan: Initializing renderer {}x{}", config.width, config.height);
        
        let backend = backend::VulkanBackend::new(native_window, &config)?;
        let frame_data = Self::create_frame_data(&backend)?;
        
        Ok(Self {
            backend,
            frame_data,
            current_frame: 0,
        })
    }
    
    /// Begin frame - returns vertex slice for rendering
    pub fn begin_frame(&mut self) -> K1Result<FrameContext> {
        let frame = self.current_frame;
        let frame_data = &mut self.frame_data[frame];
        
        // Wait for previous frame
        unsafe {
            self.backend.device.wait_for_fences(
                &[frame_data.in_flight],
                true,
                u64::MAX,
            ).map_err(|_| K1Error::DeviceLost)?;
            
            self.backend.device.reset_fences(&[frame_data.in_flight])
                .map_err(|_| K1Error::DeviceLost)?;
        }
        
        // Reset vertex count
        frame_data.vertex_count = 0;
        
        // Reset command buffer
        unsafe {
            self.backend.device.reset_command_buffer(
                frame_data.command_buffer,
                vk::CommandBufferResetFlags::empty(),
            ).map_err(|_| K1Error::DeviceLost)?;
            
            self.backend.device.begin_command_buffer(
                frame_data.command_buffer,
                &vk::CommandBufferBeginInfo::default()
                    .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT),
            ).map_err(|_| K1Error::DeviceLost)?;
        }
        
        // Begin render pass
        self.backend.begin_render_pass(frame_data.command_buffer);
        
        Ok(FrameContext {
            vertices: unsafe {
                std::slice::from_raw_parts_mut(
                    frame_data.vertex_mapped,
                    MAX_VERTICES_PER_FRAME,
                )
            },
            vertex_count: &mut frame_data.vertex_count,
        })
    }
    
    /// End frame and present
    pub fn end_frame(&mut self) -> K1Result<()> {
        let frame = self.current_frame;
        let frame_data = &self.frame_data[frame];
        
        // End render pass
        unsafe {
            self.backend.device.cmd_end_render_pass(frame_data.command_buffer);
            self.backend.device.end_command_buffer(frame_data.command_buffer)
                .map_err(|_| K1Error::DeviceLost)?;
        }
        
        // Submit
        let wait_stages = [vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
        let submit_info = vk::SubmitInfo::default()
            .wait_semaphores(&[frame_data.image_available])
            .wait_dst_stage_mask(&wait_stages)
            .command_buffers(std::slice::from_ref(&frame_data.command_buffer))
            .signal_semaphores(&[frame_data.render_finished]);
        
        unsafe {
            self.backend.device.queue_submit(
                self.backend.graphics_queue,
                std::slice::from_ref(&submit_info),
                frame_data.in_flight,
            ).map_err(|_| K1Error::DeviceLost)?;
        }
        
        // Present
        self.backend.present(frame_data.render_finished)?;
        
        self.current_frame = (frame + 1) % MAX_FRAMES_IN_FLIGHT;
        
        Ok(())
    }
    
    fn create_frame_data(backend: &backend::VulkanBackend) -> K1Result<[FrameData; MAX_FRAMES_IN_FLIGHT]> {
        // Implementation creates buffers, command pools, sync primitives
        todo!("Implement frame data creation")
    }
}

/// Context for rendering a single frame
pub struct FrameContext<'a> {
    pub vertices: &'a mut [Vertex],
    pub vertex_count: &'a mut usize,
}

impl<'a> FrameContext<'a> {
    /// Add vertices to frame
    pub fn push_vertices(&mut self, verts: &[Vertex]) -> K1Result<()> {
        let start = *self.vertex_count;
        if start + verts.len() > self.vertices.len() {
            return Err(K1Error::OutOfMemory);
        }
        self.vertices[start..start + verts.len()].copy_from_slice(verts);
        *self.vertex_count += verts.len();
        Ok(())
    }
}
