use crate::vertex::Vertex;
use cgmath::Point3;
use serde::{Deserialize, Serialize};
use vulkanalia::prelude::v1_0::*;

#[derive(Debug)]
pub(crate) struct Mesh {
    pub(crate) vertex_buffer: vk::Buffer,
    pub(crate) vertex_buffer_memory: vk::DeviceMemory,
    pub(crate) index_buffer: vk::Buffer,
    pub(crate) index_buffer_memory: vk::DeviceMemory,
    pub(crate) index_count: u32,
    // TODO make instances pos, rot, scale a separate struct
    pub(crate) instances_positions: Vec<Point3<f32>>,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct SerializedMesh {
    pub(crate) vertices: Vec<Vertex>,
    pub(crate) indices: Vec<u32>,
}
