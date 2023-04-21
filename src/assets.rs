use vulkanalia::prelude::v1_0::*;
use vulkanalia::vk::DeviceV1_0;
use vulkanalia::Device;

use std::collections::HashMap;

#[derive(Debug)]
pub(crate) struct Assets {
    pub(crate) meshes: HashMap<String, Mesh>,
}

impl Default for Assets {
    fn default() -> Self {
        Self {
            meshes: HashMap::new(),
        }
    }
}

#[derive(Debug)]
pub(crate) struct Texture {
    // TODO
}

#[derive(Debug)]
pub(crate) struct Mesh {
    pub(crate) vertex_buffer: vk::Buffer,
    pub(crate) vertex_buffer_memory: vk::DeviceMemory,
    pub(crate) index_buffer: vk::Buffer,
    pub(crate) index_buffer_memory: vk::DeviceMemory,
    pub(crate) instances: Vec<cgmath::Point3<f32>>,
    pub(crate) index_count: u32,
}

impl Mesh {
    fn drop(&mut self, device: &Device) {
        unsafe {
            device.destroy_buffer(self.index_buffer, None);
            device.free_memory(self.index_buffer_memory, None);
            device.destroy_buffer(self.vertex_buffer, None);
            device.free_memory(self.vertex_buffer_memory, None);
        }
    }
}
