use anyhow::Result;

use vulkanalia::prelude::v1_0::*;

use crate::{
    app::AppData,
    vertex::InstanceData,
    vertex_buffer::{copy_buffer, create_buffer},
};

pub(crate) unsafe fn create_instance_buffer(
    instance_data: &InstanceData,
    instance: &Instance,
    device: &Device,
    data: &mut AppData,
) -> Result<(vk::Buffer, vk::DeviceMemory)> {
    let size =
        (std::mem::size_of::<cgmath::Matrix4<f32>>() * instance_data.model_matrix.len()) as u64;

    let (staging_buffer, staging_buffer_memory) = create_buffer(
        instance,
        device,
        data,
        size,
        vk::BufferUsageFlags::TRANSFER_SRC,
        vk::MemoryPropertyFlags::HOST_COHERENT | vk::MemoryPropertyFlags::HOST_VISIBLE,
    )?;

    let memory = device.map_memory(staging_buffer_memory, 0, size, vk::MemoryMapFlags::empty())?;

    std::ptr::copy_nonoverlapping(
        instance_data.model_matrix.as_ptr(),
        memory.cast(),
        instance_data.model_matrix.len(),
    );

    device.unmap_memory(staging_buffer_memory);

    let (instance_buffer, instance_buffer_memory) = create_buffer(
        instance,
        device,
        data,
        size,
        vk::BufferUsageFlags::TRANSFER_DST | vk::BufferUsageFlags::VERTEX_BUFFER,
        vk::MemoryPropertyFlags::DEVICE_LOCAL,
    )?;

    copy_buffer(device, data, staging_buffer, instance_buffer, size)?;
    device.destroy_buffer(staging_buffer, None);
    device.free_memory(staging_buffer_memory, None);

    Ok((instance_buffer, instance_buffer_memory))
}
