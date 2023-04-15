use vulkanalia::prelude::v1_0::*;

use lazy_static::lazy_static;
lazy_static! {
    pub(crate) static ref VERTICES: Vec<Vertex> = vec![
        Vertex::new(
            cgmath::vec3(-0.5, -0.5, 0.0),
            cgmath::vec3(1.0, 0.0, 0.0),
            cgmath::vec2(1.0, 0.0)
        ),
        Vertex::new(
            cgmath::vec3(0.5, -0.5, 0.0),
            cgmath::vec3(0.0, 1.0, 0.0),
            cgmath::vec2(0.0, 0.0)
        ),
        Vertex::new(
            cgmath::vec3(0.5, 0.5, 0.0),
            cgmath::vec3(0.0, 0.0, 1.0),
            cgmath::vec2(0.0, 1.0)
        ),
        Vertex::new(
            cgmath::vec3(-0.5, 0.5, 0.0),
            cgmath::vec3(1.0, 1.0, 1.0),
            cgmath::vec2(1.0, 1.0)
        ),
        Vertex::new(
            cgmath::vec3(-0.5, -0.5, -0.5),
            cgmath::vec3(1.0, 0.0, 0.0),
            cgmath::vec2(1.0, 0.0)
        ),
        Vertex::new(
            cgmath::vec3(0.5, -0.5, -0.5),
            cgmath::vec3(0.0, 1.0, 0.0),
            cgmath::vec2(0.0, 0.0)
        ),
        Vertex::new(
            cgmath::vec3(0.5, 0.5, -0.5),
            cgmath::vec3(0.0, 0.0, 1.0),
            cgmath::vec2(0.0, 1.0)
        ),
        Vertex::new(
            cgmath::vec3(-0.5, 0.5, -0.5),
            cgmath::vec3(1.0, 1.0, 1.0),
            cgmath::vec2(1.0, 1.0)
        ),
    ];
}
// OPTIMIZE if there is more than 65,536 unique vertices use u32.
pub(crate) const INDICES: &[u16] = &[0, 1, 2, 2, 3, 0, 4, 5, 6, 6, 7, 4];

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub(crate) struct Vertex {
    pos: cgmath::Vector3<f32>,
    color: cgmath::Vector3<f32>,
    tex_coord: cgmath::Vector2<f32>,
}

impl Vertex {
    pub(crate) fn new(
        pos: cgmath::Vector3<f32>,
        color: cgmath::Vector3<f32>,
        tex_coord: cgmath::Vector2<f32>,
    ) -> Self {
        Self {
            pos,
            color,
            tex_coord,
        }
    }

    pub(crate) fn binding_description() -> vk::VertexInputBindingDescription {
        vk::VertexInputBindingDescription::builder()
            .binding(0)
            .stride(std::mem::size_of::<Vertex>() as u32)
            .input_rate(vk::VertexInputRate::VERTEX)
            .build()
    }

    pub(crate) fn attribute_descriptions() -> [vk::VertexInputAttributeDescription; 3] {
        let pos = vk::VertexInputAttributeDescription::builder()
            .binding(0)
            .location(0)
            .format(vk::Format::R32G32B32_SFLOAT)
            .offset(0)
            .build();

        let color_offset = std::mem::size_of::<cgmath::Vector3<f32>>() as u32;
        let color = vk::VertexInputAttributeDescription::builder()
            .binding(0)
            .location(1)
            .format(vk::Format::R32G32B32_SFLOAT)
            .offset(color_offset)
            .build();

        let tex_coord_offset = color_offset + std::mem::size_of::<cgmath::Vector3<f32>>() as u32;
        let tex_coord = vk::VertexInputAttributeDescription::builder()
            .binding(0)
            .location(2)
            .format(vk::Format::R32G32_SFLOAT)
            .offset(tex_coord_offset)
            .build();

        [pos, color, tex_coord]
    }
}
