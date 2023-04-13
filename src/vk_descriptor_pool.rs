use anyhow::Result;

use vulkanalia::prelude::v1_0::*;

use crate::{app::AppData, vk_uniform_buffer::UniformBufferObject};

pub(crate) unsafe fn create_descriptor_pool(device: &Device, data: &mut AppData) -> Result<()> {
    let ubo_size = vk::DescriptorPoolSize::builder()
        .type_(vk::DescriptorType::UNIFORM_BUFFER)
        .descriptor_count(data.swapchain_images.len() as u32);

    let pool_sizes = &[ubo_size];
    let info = vk::DescriptorPoolCreateInfo::builder()
        .pool_sizes(pool_sizes)
        .max_sets(data.swapchain_images.len() as u32);

    data.descriptor_pool = device.create_descriptor_pool(&info, None)?;

    Ok(())
}

pub(crate) unsafe fn create_descriptor_sets(device: &Device, data: &mut AppData) -> Result<()> {
    let layouts = vec![data.descriptor_set_layout; data.swapchain_images.len()];
    let info = vk::DescriptorSetAllocateInfo::builder()
        .descriptor_pool(data.descriptor_pool)
        .set_layouts(&layouts);

    data.descriptor_sets = device.allocate_descriptor_sets(&info)?;

    for i in 0..data.swapchain_images.len() {
        let info = vk::DescriptorBufferInfo::builder()
            .buffer(data.uniform_buffers[i])
            .offset(0)
            .range(std::mem::size_of::<UniformBufferObject>() as u64);

        let buffer_info = &[info];
        let ubo_write = vk::WriteDescriptorSet::builder()
            .dst_set(data.descriptor_sets[i])
            .dst_binding(0)
            .dst_array_element(0)
            .descriptor_type(vk::DescriptorType::UNIFORM_BUFFER)
            .buffer_info(buffer_info);

        device.update_descriptor_sets(&[ubo_write], &[] as &[vk::CopyDescriptorSet]);
    }

    Ok(())
}
