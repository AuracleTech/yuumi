use anyhow::{anyhow, Result};

use winit::window::Window;

use vulkanalia::loader::{LibloadingLoader, LIBRARY};
use vulkanalia::prelude::v1_0::*;
use vulkanalia::vk::ExtDebugUtilsExtension;

use crate::{vk_instance, vk_physical, VALIDATION_ENABLED};

/// Our Vulkan app.
#[derive(Clone, Debug)]
pub(crate) struct App {
    entry: Entry,
    instance: Instance,
    data: AppData,
    pub(crate) destroying: bool,
    pub(crate) physical_device: vk::PhysicalDevice,
}

impl App {
    /// Creates our Vulkan app.
    pub(crate) unsafe fn create(window: &Window) -> Result<Self> {
        let loader = LibloadingLoader::new(LIBRARY)?;
        let entry = Entry::new(loader).map_err(|b| anyhow!("{}", b))?;
        let mut data = AppData::default();
        let instance = vk_instance::create_instance(window, &entry, &mut data)?;
        let physical_device = vk_physical::pick_physical_device(&instance)?;
        Ok(Self {
            entry,
            instance,
            data,
            destroying: false,
            physical_device,
        })
    }

    /// Renders a frame for our Vulkan app.
    pub(crate) unsafe fn render(&mut self, window: &Window) -> Result<()> {
        Ok(())
    }

    /// Destroys our Vulkan app.
    pub(crate) unsafe fn destroy(&mut self) {
        if VALIDATION_ENABLED {
            self.instance
                .destroy_debug_utils_messenger_ext(self.data.messenger, None);
        }

        self.instance.destroy_instance(None);
    }
}

/// The Vulkan handles and associated properties used by our Vulkan app.
#[derive(Clone, Debug, Default)]
pub(crate) struct AppData {
    pub(crate) messenger: vk::DebugUtilsMessengerEXT,
}
