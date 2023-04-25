use cgmath::Point3;
use vulkanalia::prelude::v1_0::*;

use std::collections::HashMap;

#[derive(Debug)]
pub(crate) struct Assets {
    pub(crate) meshes: HashMap<String, Mesh>,
    pub(crate) textures: HashMap<String, Texture>,
}

#[derive(Debug)]
pub(crate) struct Texture {
    pub image: vk::Image,
    pub image_view: vk::ImageView,
    pub image_memory: vk::DeviceMemory,
    pub _width: u32,
    pub _height: u32,
    pub _mip_levels: u32,
    // OPTIMIZE use a reference to the image view to reuse the same image view for multiple textures
    pub _format: vk::Format,
    // OPTIMIZE use a reference to the texture sampler to reuse the same sampler for multiple textures
    pub sampler: vk::Sampler,
}

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
