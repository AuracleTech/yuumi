use anyhow::Result;

use vulkanalia::{prelude::v1_0::*, vk::Sampler};

use crate::app::AppData;

pub(crate) unsafe fn create_texture_sampler(
    device: &Device,
    data: &mut AppData,
    mip_levels: &u32,
) -> Result<Sampler> {
    let info = vk::SamplerCreateInfo::builder()
        .mag_filter(vk::Filter::LINEAR)
        .min_filter(vk::Filter::LINEAR)
        .address_mode_u(vk::SamplerAddressMode::REPEAT)
        .address_mode_v(vk::SamplerAddressMode::REPEAT)
        .address_mode_w(vk::SamplerAddressMode::REPEAT)
        .anisotropy_enable(data.setting_anisotropy)
        .max_anisotropy(data.setting_max_sampler_anisotropy)
        .border_color(vk::BorderColor::INT_OPAQUE_BLACK)
        .unnormalized_coordinates(false)
        .compare_enable(false)
        .compare_op(vk::CompareOp::ALWAYS)
        .mipmap_mode(vk::SamplerMipmapMode::LINEAR)
        .min_lod(0.0) // Optional
        .max_lod(*mip_levels as f32)
        .mip_lod_bias(0.0); // Optional

    Ok(device.create_sampler(&info, None)?)
}
