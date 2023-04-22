use cgmath::Point3;
use vulkanalia::prelude::v1_0::*;

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
    pub(crate) instances_positions: Vec<Point3<f32>>,
    pub(crate) index_count: u32,
}
