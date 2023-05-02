use std::hash::Hash;
use std::hash::Hasher;

use serde::Deserialize;
use serde::Serialize;
use vulkanalia::prelude::v1_0::*;

#[repr(C)]
#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub(crate) struct Vertex {
    pub(crate) pos: cgmath::Vector3<f32>,
    pub(crate) color: cgmath::Vector3<f32>,
    pub(crate) tex_coord: cgmath::Vector2<f32>,
}

impl PartialEq for Vertex {
    fn eq(&self, other: &Self) -> bool {
        self.pos == other.pos && self.color == other.color && self.tex_coord == other.tex_coord
    }
}

impl Eq for Vertex {}

impl Hash for Vertex {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.pos[0].to_bits().hash(state);
        self.pos[1].to_bits().hash(state);
        self.pos[2].to_bits().hash(state);
        self.color[0].to_bits().hash(state);
        self.color[1].to_bits().hash(state);
        self.color[2].to_bits().hash(state);
        self.tex_coord[0].to_bits().hash(state);
        self.tex_coord[1].to_bits().hash(state);
    }
}

impl Vertex {
    pub(crate) fn binding_description() -> vk::VertexInputBindingDescription {
        vk::VertexInputBindingDescription::builder()
            .binding(0)
            .stride(std::mem::size_of::<Vertex>() as u32)
            .input_rate(vk::VertexInputRate::VERTEX)
            .build()
    }

    pub(crate) fn attribute_descriptions() -> Vec<vk::VertexInputAttributeDescription> {
        let pos_size = std::mem::size_of::<cgmath::Vector3<f32>>();
        let pos = vk::VertexInputAttributeDescription::builder()
            .binding(0)
            .location(0)
            .format(vk::Format::R32G32B32_SFLOAT)
            .offset(0)
            .build();

        let color_size = std::mem::size_of::<cgmath::Vector3<f32>>();
        let color = vk::VertexInputAttributeDescription::builder()
            .binding(0)
            .location(1)
            .format(vk::Format::R32G32B32_SFLOAT)
            .offset(pos_size as u32)
            .build();

        let _tex_coord_size = std::mem::size_of::<cgmath::Vector2<f32>>();
        let tex_coord = vk::VertexInputAttributeDescription::builder()
            .binding(0)
            .location(2)
            .format(vk::Format::R32G32_SFLOAT)
            .offset((pos_size + color_size) as u32)
            .build();

        vec![pos, color, tex_coord]
    }
}

#[repr(C)]
pub struct InstanceData {
    pub model_matrix: Vec<cgmath::Matrix4<f32>>,
}

impl InstanceData {
    pub(crate) fn binding_description() -> vk::VertexInputBindingDescription {
        vk::VertexInputBindingDescription::builder()
            .binding(1)
            .stride(std::mem::size_of::<cgmath::Matrix4<f32>>() as u32)
            .input_rate(vk::VertexInputRate::INSTANCE)
            .build()
    }

    pub(crate) fn attribute_descriptions() -> Vec<vk::VertexInputAttributeDescription> {
        let instance_transform0 = vk::VertexInputAttributeDescription::builder()
            .binding(1)
            .location(3)
            .format(vk::Format::R32G32B32A32_SFLOAT)
            .offset(0)
            .build();

        let instance_transform1 = vk::VertexInputAttributeDescription::builder()
            .binding(1)
            .location(4)
            .format(vk::Format::R32G32B32A32_SFLOAT)
            .offset(16)
            .build();

        let instance_transform2 = vk::VertexInputAttributeDescription::builder()
            .binding(1)
            .location(5)
            .format(vk::Format::R32G32B32A32_SFLOAT)
            .offset(32)
            .build();

        let instance_transform3 = vk::VertexInputAttributeDescription::builder()
            .binding(1)
            .location(6)
            .format(vk::Format::R32G32B32A32_SFLOAT)
            .offset(48)
            .build();

        vec![
            instance_transform0,
            instance_transform1,
            instance_transform2,
            instance_transform3,
        ]
    }
}
