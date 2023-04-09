use anyhow::{anyhow, Result};

use winit::window::Window;

use vulkanalia::loader::{LibloadingLoader, LIBRARY};
use vulkanalia::prelude::v1_0::*;
use vulkanalia::vk::{ExtDebugUtilsExtension, KhrSurfaceExtension, KhrSwapchainExtension};
use vulkanalia::window::create_surface;
use vulkanalia::Device;

use crate::vk_framebuffer::create_framebuffers;
use crate::vk_pipeline::create_pipeline;
use crate::vk_render_pass::create_render_pass;
use crate::vk_swapchain::{create_swapchain, create_swapchain_image_views};
use crate::{vk_instance, vk_logical_device, vk_physical_device, VALIDATION_ENABLED};

/// Our Vulkan app.
#[derive(Clone, Debug)]
pub(crate) struct App {
    _entry: Entry,
    instance: Instance,
    data: AppData,
    device: Device,
    pub(crate) destroying: bool,
}

impl App {
    /// Creates our Vulkan app.
    pub(crate) unsafe fn create(window: &Window) -> Result<Self> {
        let loader = LibloadingLoader::new(LIBRARY)?;
        let _entry = Entry::new(loader).map_err(|b| anyhow!("{}", b))?;
        let mut data = AppData::default();
        let instance = vk_instance::create_instance(window, &_entry, &mut data)?;
        data.surface = create_surface(&instance, &window, &window)?;
        vk_physical_device::pick_physical_device(&instance, &mut data)?;
        let device = vk_logical_device::create_logical_device(&instance, &mut data)?;
        create_swapchain(window, &instance, &device, &mut data)?;
        create_swapchain_image_views(&device, &mut data)?;
        create_render_pass(&device, &mut data)?;
        let layout_info = vk::PipelineLayoutCreateInfo::builder();
        data.pipeline_layout = device.create_pipeline_layout(&layout_info, None)?;
        create_pipeline(&device, &mut data)?;
        create_framebuffers(&device, &mut data)?;
        Ok(Self {
            _entry,
            instance,
            data,
            device,
            destroying: false,
        })
    }

    /// Renders a frame for our Vulkan app.
    pub(crate) unsafe fn render(&mut self, _window: &Window) -> Result<()> {
        Ok(())
    }

    /// Destroys our Vulkan app.
    pub(crate) unsafe fn destroy(&mut self) {
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
        self.device.destroy_device(None);

        if VALIDATION_ENABLED {
            self.instance
                .destroy_debug_utils_messenger_ext(self.data.messenger, None);
        }
        self.instance.destroy_surface_khr(self.data.surface, None);
        self.instance.destroy_instance(None);
    }
}

/// The Vulkan handles and associated properties used by our Vulkan app.
#[derive(Clone, Debug, Default)]
pub(crate) struct AppData {
    pub(crate) surface: vk::SurfaceKHR,
    pub(crate) messenger: vk::DebugUtilsMessengerEXT,
    pub(crate) physical_device: vk::PhysicalDevice,
    pub(crate) graphics_queue: vk::Queue,
    pub(crate) present_queue: vk::Queue,
    pub(crate) swapchain_format: vk::Format,
    pub(crate) swapchain_extent: vk::Extent2D,
    pub(crate) swapchain: vk::SwapchainKHR,
    pub(crate) swapchain_images: Vec<vk::Image>,
    pub(crate) swapchain_image_views: Vec<vk::ImageView>,
    pub(crate) render_pass: vk::RenderPass,
    pub(crate) pipeline_layout: vk::PipelineLayout,
    pub(crate) pipeline: vk::Pipeline,
    pub(crate) framebuffers: Vec<vk::Framebuffer>,
}
