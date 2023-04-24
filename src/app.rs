use std::collections::HashMap;

use anyhow::{anyhow, Result};

use cgmath::EuclideanSpace;
use winit::window::Window;

use vulkanalia::loader::{LibloadingLoader, LIBRARY};
use vulkanalia::prelude::v1_0::*;
use vulkanalia::vk::{ExtDebugUtilsExtension, KhrSurfaceExtension, KhrSwapchainExtension};
use vulkanalia::window::create_surface;
use vulkanalia::Device;

use crate::assets::{Assets, Mesh, Texture};
use crate::command_buffer::{create_command_buffers, create_command_pools};
use crate::depth_object::create_depth_objects;
use crate::descriptor_layout::create_descriptor_set_layout;
use crate::descriptor_pool::{create_descriptor_pool, create_descriptor_sets};
use crate::framebuffer::create_framebuffers;
use crate::image_view::{create_image_view, create_swapchain_image_views};
use crate::instance::create_instance;
use crate::logical_device::create_logical_device;
use crate::metrics::Metrics;
use crate::msaa::create_color_objects;
use crate::physical_device::pick_physical_device;
use crate::pipeline::create_pipeline;
use crate::render_pass::create_render_pass;
use crate::swapchain::create_swapchain;
use crate::sync_object::create_sync_objects;
use crate::texture_image::create_texture_image;
use crate::texture_sampler::create_texture_sampler;
use crate::uniform_buffer::{create_uniform_buffers, UniformBufferObject};
use crate::vertex::Vertex;
use crate::vertex_buffer::{create_index_buffer, create_vertex_buffer};

pub(crate) const MAX_FRAMES_IN_FLIGHT: usize = 2;

pub(crate) const VALIDATION_ENABLED: bool = cfg!(debug_assertions);

pub(crate) const VALIDATION_LAYER: vk::ExtensionName =
    vk::ExtensionName::from_bytes(b"VK_LAYER_KHRONOS_validation");

#[derive(Debug)]
pub(crate) struct VulkanApp {
    _entry: Entry,
    instance: Instance,
    data: AppData,
    pub(crate) device: Device,
    pub(crate) destroying: bool,
    pub(crate) frame: usize,
    pub(crate) resized: bool,
    pub(crate) minimized: bool,
    pub(crate) metrics: Metrics,
    pub(crate) assets: Assets,
}

impl VulkanApp {
    pub(crate) fn new(window: &Window) -> Result<Self> {
        unsafe {
            let loader = LibloadingLoader::new(LIBRARY)?;
            let _entry = Entry::new(loader).map_err(|b| anyhow!("{}", b))?;
            let mut data: AppData = AppData::default();
            let instance = create_instance(&window, &_entry, &mut data)?;
            data.surface = create_surface(&instance, &window, &window)?;
            pick_physical_device(&instance, &mut data)?;
            let device = create_logical_device(&instance, &mut data)?;
            let mut app = Self {
                _entry,
                instance,
                data,
                device,
                destroying: false,
                frame: 0,
                resized: false,
                minimized: false,
                metrics: Metrics::default(),
                assets: Assets {
                    meshes: HashMap::new(),
                    textures: HashMap::new(),
                },
            };
            create_swapchain(&window, &app.instance, &app.device, &mut app.data)?;
            create_swapchain_image_views(&app.device, &mut app.data)?;
            create_render_pass(&app.instance, &app.device, &mut app.data)?;
            create_descriptor_set_layout(&app.device, &mut app.data)?;
            create_pipeline(&app.device, &mut app.data)?;
            create_command_pools(&app.instance, &app.device, &mut app.data)?;
            create_color_objects(&app.instance, &app.device, &mut app.data)?;
            create_depth_objects(&app.instance, &app.device, &mut app.data)?;
            create_framebuffers(&app.device, &mut app.data)?;
            app.load_model_obj("viking_room")?;
            app.load_texture_png("viking_room")?;
            let texture = app
                .assets
                .textures
                .get("viking_room")
                .expect("Texture not found");

            create_uniform_buffers(&app.instance, &app.device, &mut app.data)?;
            create_descriptor_pool(&app.device, &mut app.data)?;
            create_descriptor_sets(
                &app.device,
                &mut app.data,
                &texture.image_view,
                &texture.sampler,
            )?;
            create_command_buffers(&app.device, &mut app.data)?;
            create_sync_objects(&app.device, &mut app.data)?;

            app.metrics.cycle.start();

            Ok(app)
        }
    }

    pub(crate) unsafe fn render(&mut self, window: &Window) -> Result<()> {
        self.metrics.cycle.start_frame();

        // We wait for the fence of the current frame to finish executing. This is because we're going to re-use this frame's resources.
        self.device.wait_for_fences(
            &[self.data.in_flight_fences[self.frame]],
            true,
            u64::max_value(),
        )?;

        // We acquire the next image from the swapchain.
        let result = self.device.acquire_next_image_khr(
            self.data.swapchain,
            u64::max_value(),
            self.data.image_available_semaphores[self.frame],
            vk::Fence::null(),
        );

        // We check if the swapchain is out of date. If it is, we recreate it.
        let image_index = match result {
            Ok((image_index, _)) => image_index as usize,
            Err(vk::ErrorCode::OUT_OF_DATE_KHR) => return self.recreate_swapchain(window),
            Err(e) => return Err(anyhow!(e)),
        };

        // We check if the image is in use. If it is, we wait for it to finish.
        if !self.data.images_in_flight[image_index as usize].is_null() {
            self.device.wait_for_fences(
                &[self.data.images_in_flight[image_index as usize]],
                true,
                u64::max_value(),
            )?;
        }

        // We mark the image as in use.
        self.data.images_in_flight[image_index as usize] = self.data.in_flight_fences[self.frame];

        // We update the command buffer.
        self.update_command_buffer(image_index)?;

        // We update the uniform buffer.
        self.update_uniform_buffer(image_index)?;

        // We build the submit info that we're going to use to submit to the graphics queue.
        let wait_semaphores = &[self.data.image_available_semaphores[self.frame]];
        let wait_stages = &[vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
        let command_buffers = &[self.data.command_buffers[image_index as usize]];
        let signal_semaphores = &[self.data.render_finished_semaphores[self.frame]];
        let submit_info = vk::SubmitInfo::builder()
            .wait_semaphores(wait_semaphores)
            .wait_dst_stage_mask(wait_stages)
            .command_buffers(command_buffers)
            .signal_semaphores(signal_semaphores)
            .build();

        // We reset the fence of the current frame. This is because we're going to re-use this frame's resources.
        self.device
            .reset_fences(&[self.data.in_flight_fences[self.frame]])?;

        // We submit the command buffer to the graphics queue.
        self.device.queue_submit(
            self.data.graphics_queue,
            &[submit_info],
            self.data.in_flight_fences[self.frame],
        )?;

        // We build the present info that we're going to use to present.
        let swapchains = &[self.data.swapchain];
        let image_indices = &[image_index as u32];
        let present_info = vk::PresentInfoKHR::builder()
            .wait_semaphores(signal_semaphores)
            .swapchains(swapchains)
            .image_indices(image_indices)
            .build();

        //  We present the image to the screen.
        let result = self
            .device
            .queue_present_khr(self.data.present_queue, &present_info);

        // We check if the swapchain is suboptimal. If it is, we recreate it.
        let changed = result == Ok(vk::SuccessCode::SUBOPTIMAL_KHR)
            || result == Err(vk::ErrorCode::OUT_OF_DATE_KHR);

        // We check if the window was resized. If it was, we recreate the swapchain.
        if self.resized || changed {
            self.recreate_swapchain(window)?;
            self.resized = false;
        } else if let Err(e) = result {
            return Err(anyhow!(e));
        }

        // We increment the frame index.
        self.frame = (self.frame + 1) % MAX_FRAMES_IN_FLIGHT;

        self.metrics.cycle.end_frame();
        self.metrics.total_frames += 1;

        Ok(())
    }

    unsafe fn update_command_buffer(&mut self, image_index: usize) -> Result<()> {
        let command_pool = self.data.command_pools[image_index];
        self.device
            .reset_command_pool(command_pool, vk::CommandPoolResetFlags::empty())?;

        let command_buffer = self.data.command_buffers[image_index];

        let info = vk::CommandBufferBeginInfo::builder()
            .flags(vk::CommandBufferUsageFlags::ONE_TIME_SUBMIT);

        self.device.begin_command_buffer(command_buffer, &info)?;

        let render_area = vk::Rect2D::builder()
            .offset(vk::Offset2D::default())
            .extent(self.data.swapchain_extent);

        let color_clear_value = vk::ClearValue {
            color: vk::ClearColorValue {
                float32: [0.0, 0.0, 0.0, 1.0],
            },
        };

        let depth_clear_value = vk::ClearValue {
            depth_stencil: vk::ClearDepthStencilValue {
                depth: 1.0,
                stencil: 0,
            },
        };

        let clear_values = &[color_clear_value, depth_clear_value];
        let info = vk::RenderPassBeginInfo::builder()
            .render_pass(self.data.render_pass)
            .framebuffer(self.data.framebuffers[image_index])
            .render_area(render_area)
            .clear_values(clear_values);

        self.device.cmd_begin_render_pass(
            command_buffer,
            &info,
            vk::SubpassContents::SECONDARY_COMMAND_BUFFERS,
        );

        let mut secondary_command_buffers = Vec::new();

        let secondary_command_buffer = self.update_secondary_command_buffer(image_index)?;
        secondary_command_buffers.push(secondary_command_buffer);

        self.device
            .cmd_execute_commands(command_buffer, &secondary_command_buffers);

        self.device.cmd_end_render_pass(command_buffer);

        self.device.end_command_buffer(command_buffer)?;

        Ok(())
    }

    unsafe fn update_secondary_command_buffer(
        &mut self,
        image_index: usize,
    ) -> Result<vk::CommandBuffer> {
        // Allocate
        let secondary_command_buffers = &mut self.data.secondary_command_buffers[image_index];

        let secondary_command_buffer_index = 1;
        while secondary_command_buffer_index >= secondary_command_buffers.len() {
            let allocate_info = vk::CommandBufferAllocateInfo::builder()
                .command_pool(self.data.command_pools[image_index])
                .level(vk::CommandBufferLevel::SECONDARY)
                .command_buffer_count(1);

            let command_buffer = self.device.allocate_command_buffers(&allocate_info)?[0];
            secondary_command_buffers.push(command_buffer);
        }

        let secondary_command_buffer = secondary_command_buffers[secondary_command_buffer_index];

        // Get model
        let viking_room_mesh = self
            .assets
            .meshes
            .get("viking_room")
            .expect("Model not found");

        // Commands

        let inheritance_info = vk::CommandBufferInheritanceInfo::builder()
            .render_pass(self.data.render_pass)
            .subpass(0)
            .framebuffer(self.data.framebuffers[image_index]);

        let info = vk::CommandBufferBeginInfo::builder()
            .flags(vk::CommandBufferUsageFlags::RENDER_PASS_CONTINUE)
            .inheritance_info(&inheritance_info);

        self.device
            .begin_command_buffer(secondary_command_buffer, &info)?;

        self.device.cmd_bind_pipeline(
            secondary_command_buffer,
            vk::PipelineBindPoint::GRAPHICS,
            self.data.pipeline,
        );

        self.device.cmd_bind_vertex_buffers(
            secondary_command_buffer,
            0,
            &[viking_room_mesh.vertex_buffer],
            &[0],
        );
        self.device.cmd_bind_index_buffer(
            secondary_command_buffer,
            viking_room_mesh.index_buffer,
            0,
            vk::IndexType::UINT32,
        );
        self.device.cmd_bind_descriptor_sets(
            secondary_command_buffer,
            vk::PipelineBindPoint::GRAPHICS,
            self.data.pipeline_layout,
            0,
            &[self.data.descriptor_sets[image_index]],
            &[],
        );

        // Push constants

        let time = self.metrics.engine_start.elapsed().as_secs_f32();
        let rotation = cgmath::Quaternion::from(cgmath::Euler {
            x: cgmath::Deg(0.0),
            y: cgmath::Deg(0.0),
            z: cgmath::Deg(time * 5.0),
        });
        let mut index = 0;
        for instance_position in &viking_room_mesh.instances_positions {
            let model = cgmath::Matrix4::from_translation(instance_position.to_vec())
                * cgmath::Matrix4::from(rotation);
            let model_bytes = unsafe {
                std::slice::from_raw_parts(
                    &model as *const cgmath::Matrix4<f32> as *const u8,
                    std::mem::size_of::<cgmath::Matrix4<f32>>(),
                )
            };
            self.device.cmd_push_constants(
                secondary_command_buffer,
                self.data.pipeline_layout,
                vk::ShaderStageFlags::VERTEX,
                0,
                model_bytes,
            );

            let opacity = 1.0 - (index as f32 * 0.3);
            let opacity_bytes = opacity.to_ne_bytes();
            self.device.cmd_push_constants(
                secondary_command_buffer,
                self.data.pipeline_layout,
                vk::ShaderStageFlags::FRAGMENT,
                64,
                &opacity_bytes,
            );
            self.device.cmd_draw_indexed(
                secondary_command_buffer,
                viking_room_mesh.index_count as u32,
                1,
                0,
                0,
                0,
            );
            index += 1;
        }

        self.device.end_command_buffer(secondary_command_buffer)?;

        Ok(secondary_command_buffer)
    }

    unsafe fn update_uniform_buffer(&self, image_index: usize) -> Result<()> {
        let view = <cgmath::Matrix4<f32>>::look_at_rh(
            cgmath::Point3::new(6.0, 0.0, 2.0),
            cgmath::Point3::new(0.0, 0.0, 0.0),
            cgmath::Vector3::unit_z(),
        );

        let aspect_ratio =
            self.data.swapchain_extent.width as f32 / self.data.swapchain_extent.height as f32;
        let mut proj = cgmath::perspective(cgmath::Deg(45.0), aspect_ratio, 0.1, 10.0);

        proj.y.y *= -1.0;

        let ubo = UniformBufferObject { view, proj };

        // OPTIMIZE use push constants
        let memory = self.device.map_memory(
            self.data.uniform_buffers_memory[image_index],
            0,
            std::mem::size_of::<UniformBufferObject>() as u64,
            vk::MemoryMapFlags::empty(),
        )?;

        std::ptr::copy_nonoverlapping(&ubo, memory.cast(), 1);

        self.device
            .unmap_memory(self.data.uniform_buffers_memory[image_index]);

        Ok(())
    }

    pub(crate) unsafe fn load_texture_png(&mut self, name: &str) -> Result<()> {
        let path = format!("assets/models/{}.png", name);
        let image = std::fs::File::open(path)?;

        let decoder = png::Decoder::new(image);
        let mut reader = decoder.read_info()?;

        let mut pixels = vec![0; reader.info().raw_bytes()];
        reader.next_frame(&mut pixels)?;

        let size = reader.info().raw_bytes() as u64;
        let (width, height) = reader.info().size();

        let (image, image_memory, mip_levels) = create_texture_image(
            &mut self.instance,
            &mut self.device,
            &mut self.data,
            &pixels,
            size,
            width,
            height,
        )?;

        // OPTIMIZE reuse image view and sampler for the most textures possible
        let format = vk::Format::R8G8B8A8_SRGB;
        let aspects = vk::ImageAspectFlags::COLOR;
        let image_view = create_image_view(&self.device, &image, &format, &aspects, &mip_levels)?;
        let sampler = create_texture_sampler(&self.device, &mut self.data, &mip_levels)?;

        self.assets.textures.insert(
            name.to_string(),
            Texture {
                image,
                image_view,
                image_memory,
                _mip_levels: mip_levels,
                _width: width,
                _height: height,
                _format: vk::Format::R8G8B8A8_SRGB,
                sampler,
            },
        );

        Ok(())
    }

    pub(crate) fn load_model_obj(&mut self, name: &str) -> Result<()> {
        let path = format!("assets/models/{}.obj", name);
        let mut reader = std::io::BufReader::new(std::fs::File::open(path)?);

        let (models, _) = tobj::load_obj_buf(
            &mut reader,
            &tobj::LoadOptions {
                triangulate: true,
                ..Default::default()
            },
            |_| Ok(Default::default()),
        )?;

        let mut vertices = Vec::new();
        let mut indices = Vec::new();

        for model in &models {
            let mut unique_vertices = HashMap::new();
            for index in &model.mesh.indices {
                let pos_offset = (3 * index) as usize;
                let tex_coord_offset = (2 * index) as usize;

                let vertex = Vertex {
                    pos: cgmath::vec3(
                        model.mesh.positions[pos_offset],
                        model.mesh.positions[pos_offset + 1],
                        model.mesh.positions[pos_offset + 2],
                    ),
                    color: cgmath::vec3(1.0, 1.0, 1.0),
                    tex_coord: cgmath::vec2(
                        model.mesh.texcoords[tex_coord_offset],
                        1.0 - model.mesh.texcoords[tex_coord_offset + 1],
                    ),
                };

                if let Some(index) = unique_vertices.get(&vertex) {
                    indices.push(*index as u32);
                } else {
                    let index = vertices.len();
                    unique_vertices.insert(vertex, index);
                    vertices.push(vertex);
                    indices.push(index as u32);
                }
            }
        }

        unsafe {
            let (vertex_buffer, vertex_buffer_memory) = create_vertex_buffer(
                &vertices,
                &mut self.instance,
                &mut self.device,
                &mut self.data,
            )?;
            let (index_buffer, index_buffer_memory) = create_index_buffer(
                &indices,
                &mut self.instance,
                &mut self.device,
                &mut self.data,
            )?;

            let instances = vec![
                cgmath::Point3::new(0.0, -1.25, 1.0),
                cgmath::Point3::new(0.0, 1.25, 1.0),
                cgmath::Point3::new(0.0, -1.25, -1.0),
                cgmath::Point3::new(0.0, 1.25, -1.0),
            ];

            let mesh = Mesh {
                vertex_buffer,
                vertex_buffer_memory,
                index_buffer,
                index_buffer_memory,
                // TODO FLEXIBLE INSTANCES
                instances_positions: instances,
                index_count: indices.len() as u32,
            };

            self.assets.meshes.insert(name.to_string(), mesh);
        }

        Ok(())
    }

    // OPTIMIZE do not recreate swapchain if only the windows size changed
    pub(crate) unsafe fn recreate_swapchain(&mut self, window: &Window) -> Result<()> {
        self.device.device_wait_idle()?;
        self.destroy_swapchain();
        create_swapchain(window, &self.instance, &self.device, &mut self.data)?;
        create_swapchain_image_views(&self.device, &mut self.data)?;
        create_render_pass(&self.instance, &self.device, &mut self.data)?;
        create_pipeline(&self.device, &mut self.data)?;
        create_color_objects(&self.instance, &self.device, &mut self.data)?;
        create_depth_objects(&self.instance, &self.device, &mut self.data)?;
        create_framebuffers(&self.device, &mut self.data)?;
        create_uniform_buffers(&self.instance, &self.device, &mut self.data)?;
        create_descriptor_pool(&self.device, &mut self.data)?;

        let texture = self
            .assets
            .textures
            .get("viking_room")
            .expect("Texture not found");

        create_descriptor_sets(
            &self.device,
            &mut self.data,
            &texture.image_view,
            &texture.sampler,
        )?;
        create_command_buffers(&self.device, &mut self.data)?;
        self.data
            .images_in_flight
            .resize(self.data.swapchain_images.len(), vk::Fence::null());
        Ok(())
    }

    unsafe fn destroy_swapchain(&mut self) {
        self.device
            .destroy_image_view(self.data.color_image_view, None);
        self.device.free_memory(self.data.color_image_memory, None);
        self.device.destroy_image(self.data.color_image, None);
        self.device
            .destroy_image_view(self.data.depth_image_view, None);
        self.device.free_memory(self.data.depth_image_memory, None);
        self.device.destroy_image(self.data.depth_image, None);
        self.device
            .destroy_descriptor_pool(self.data.descriptor_pool, None);
        self.data
            .uniform_buffers
            .iter()
            .for_each(|b| self.device.destroy_buffer(*b, None));
        self.data
            .uniform_buffers_memory
            .iter()
            .for_each(|m| self.device.free_memory(*m, None));
        self.data
            .framebuffers
            .iter()
            .for_each(|f| self.device.destroy_framebuffer(*f, None));
        self.device.destroy_pipeline(self.data.pipeline, None);
        self.device
            .destroy_pipeline_layout(self.data.pipeline_layout, None);
        self.device.destroy_render_pass(self.data.render_pass, None);
        self.data
            .swapchain_image_views
            .iter()
            .for_each(|v| self.device.destroy_image_view(*v, None));
        self.device.destroy_swapchain_khr(self.data.swapchain, None);
    }
}

impl Drop for VulkanApp {
    fn drop(&mut self) {
        unsafe {
            self.destroy_swapchain();
            self.data
                .command_pools
                .iter()
                .for_each(|p| self.device.destroy_command_pool(*p, None));

            self.assets.textures.iter().for_each(|(_, texture)| {
                self.device.destroy_image(texture.image, None);
                self.device.free_memory(texture.image_memory, None);

                // FIX destroy samplers: self.device.destroy_sampler(texture.sampler, None);
                self.device.destroy_sampler(texture.sampler, None);
                // FIX destroy image views: self.device.destroy_image_view(texture.image_view, None);
                self.device.destroy_image_view(texture.image_view, None);
            });

            self.device
                .destroy_descriptor_set_layout(self.data.descriptor_set_layout, None);

            self.assets.meshes.iter().for_each(|(_, mesh)| {
                self.device.destroy_buffer(mesh.vertex_buffer, None);
                self.device.free_memory(mesh.vertex_buffer_memory, None);
                self.device.destroy_buffer(mesh.index_buffer, None);
                self.device.free_memory(mesh.index_buffer_memory, None);
            });

            self.data
                .in_flight_fences
                .iter()
                .for_each(|f| self.device.destroy_fence(*f, None));
            self.data
                .render_finished_semaphores
                .iter()
                .for_each(|s| self.device.destroy_semaphore(*s, None));
            self.data
                .image_available_semaphores
                .iter()
                .for_each(|s| self.device.destroy_semaphore(*s, None));
            self.device
                .destroy_command_pool(self.data.command_pool, None);
            self.device.destroy_device(None);
            self.instance.destroy_surface_khr(self.data.surface, None);

            if VALIDATION_ENABLED {
                self.instance
                    .destroy_debug_utils_messenger_ext(self.data.messenger, None);
            }

            self.instance.destroy_instance(None);
        }
    }
}

/// The Vulkan handles and associated properties used by our Vulkan app.
#[derive(Clone, Debug, Default)]
pub(crate) struct AppData {
    pub(crate) surface: vk::SurfaceKHR,
    pub(crate) messenger: vk::DebugUtilsMessengerEXT,
    pub(crate) physical_device: vk::PhysicalDevice,
    pub(crate) msaa_samples: vk::SampleCountFlags,
    pub(crate) graphics_queue: vk::Queue,
    pub(crate) present_queue: vk::Queue,
    pub(crate) swapchain_format: vk::Format,
    pub(crate) swapchain_extent: vk::Extent2D,
    pub(crate) swapchain: vk::SwapchainKHR,
    pub(crate) swapchain_images: Vec<vk::Image>,
    pub(crate) swapchain_image_views: Vec<vk::ImageView>,
    pub(crate) render_pass: vk::RenderPass,
    pub(crate) descriptor_set_layout: vk::DescriptorSetLayout,
    pub(crate) pipeline_layout: vk::PipelineLayout,
    pub(crate) pipeline: vk::Pipeline,
    pub(crate) framebuffers: Vec<vk::Framebuffer>,
    pub(crate) command_pool: vk::CommandPool,
    pub(crate) command_pools: Vec<vk::CommandPool>,
    pub(crate) command_buffers: Vec<vk::CommandBuffer>,
    pub(crate) secondary_command_buffers: Vec<Vec<vk::CommandBuffer>>,
    pub(crate) image_available_semaphores: Vec<vk::Semaphore>,
    pub(crate) render_finished_semaphores: Vec<vk::Semaphore>,
    pub(crate) in_flight_fences: Vec<vk::Fence>,
    pub(crate) images_in_flight: Vec<vk::Fence>,
    // OPTIMIZE Use a single buffer for multiple buffers. Requires custom allocator.
    pub(crate) uniform_buffers: Vec<vk::Buffer>,
    pub(crate) uniform_buffers_memory: Vec<vk::DeviceMemory>,
    pub(crate) descriptor_pool: vk::DescriptorPool,
    pub(crate) descriptor_sets: Vec<vk::DescriptorSet>,
    pub(crate) depth_image: vk::Image,
    pub(crate) depth_image_memory: vk::DeviceMemory,
    pub(crate) depth_image_view: vk::ImageView,
    pub(crate) color_image: vk::Image,
    pub(crate) color_image_memory: vk::DeviceMemory,
    pub(crate) color_image_view: vk::ImageView,

    pub(crate) limit_max_sampler_anisotropy: f32,
    pub(crate) limit_max_push_constants_size: u32,

    pub(crate) setting_anisotropy: bool,
    pub(crate) setting_max_sampler_anisotropy: f32,
    pub(crate) setting_sample_shading: bool,
}
