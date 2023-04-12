use anyhow::Result;

use vulkanalia::prelude::v1_0::*;

use crate::{app::AppData, vertex::INDICES, vk_physical_device::QueueFamilyIndices};

pub(crate) unsafe fn create_command_pool(
    instance: &Instance,
    device: &Device,
    data: &mut AppData,
) -> Result<()> {
    let indices = QueueFamilyIndices::get(instance, data, data.physical_device)?;

    let info = vk::CommandPoolCreateInfo::builder()
        .flags(vk::CommandPoolCreateFlags::empty()) // Optional.
        .queue_family_index(indices.graphics);

    data.command_pool = device.create_command_pool(&info, None)?;

    Ok(())
}

pub(crate) unsafe fn create_command_buffers(device: &Device, data: &mut AppData) -> Result<()> {
    let allocate_info = vk::CommandBufferAllocateInfo::builder()
        .command_pool(data.command_pool)
        .level(vk::CommandBufferLevel::PRIMARY)
        .command_buffer_count(data.framebuffers.len() as u32);

    data.command_buffers = device.allocate_command_buffers(&allocate_info)?;

    let flags = vk::CommandBufferUsageFlags::SIMULTANEOUS_USE;

    for (i, command_buffer) in data.command_buffers.iter().enumerate() {
        let inheritance = vk::CommandBufferInheritanceInfo::builder();

        let info = vk::CommandBufferBeginInfo::builder()
            .flags(flags) // Optional.
            .inheritance_info(&inheritance); // Optional.

        device.begin_command_buffer(*command_buffer, &info)?;

        let render_area = vk::Rect2D::builder()
            .offset(vk::Offset2D::default())
            .extent(data.swapchain_extent);

        let color_clear_value = vk::ClearValue {
            color: vk::ClearColorValue {
                float32: [0.0, 0.0, 0.0, 1.0],
            },
        };

        let clear_values = &[color_clear_value];
        let info = vk::RenderPassBeginInfo::builder()
            .render_pass(data.render_pass)
            .framebuffer(data.framebuffers[i])
            .render_area(render_area)
            .clear_values(clear_values);

        device.cmd_begin_render_pass(*command_buffer, &info, vk::SubpassContents::INLINE);

        device.cmd_bind_pipeline(
            *command_buffer,
            vk::PipelineBindPoint::GRAPHICS,
            data.pipeline,
        );

        device.cmd_bind_vertex_buffers(*command_buffer, 0, &[data.vertex_buffer], &[0]);
        device.cmd_bind_index_buffer(*command_buffer, data.index_buffer, 0, vk::IndexType::UINT16);
        device.cmd_draw_indexed(*command_buffer, INDICES.len() as u32, 1, 0, 0, 0);

        device.cmd_end_render_pass(*command_buffer);

        device.end_command_buffer(*command_buffer)?;
    }

    Ok(())
}
