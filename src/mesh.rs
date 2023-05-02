use crate::vertex::Vertex;
use serde::{Deserialize, Serialize};
use vulkanalia::prelude::v1_0::*;

#[derive(Debug)]
pub(crate) struct Mesh {
    pub(crate) vertex_buffer: vk::Buffer,
    pub(crate) vertex_buffer_memory: vk::DeviceMemory,
    pub(crate) index_count: u32,
    pub(crate) index_buffer: vk::Buffer,
    pub(crate) index_buffer_memory: vk::DeviceMemory,
    pub(crate) instance_count: u32,
    pub(crate) instance_buffer: vk::Buffer,
    pub(crate) instance_buffer_memory: vk::DeviceMemory,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct SerializedMesh {
    pub(crate) vertices: Vec<Vertex>,
    pub(crate) indices: Vec<u32>,
}
