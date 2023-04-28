use anyhow::{anyhow, Result};

use vulkanalia::prelude::v1_0::*;
use vulkanalia::Device;

pub(crate) unsafe fn create_shader_module(
    device: &Device,
    name: String,
) -> Result<vk::ShaderModule> {
    let path = format!("lib/{}.spv", name);
    let file_content = std::fs::read(path)?;

    let bytecode = Vec::<u8>::from(file_content);
    let (prefix, code, suffix) = bytecode.align_to::<u32>();
    if !prefix.is_empty() || !suffix.is_empty() {
        return Err(anyhow!("Shader bytecode is not properly aligned."));
    }

    let info = vk::ShaderModuleCreateInfo::builder()
        .code_size(bytecode.len())
        .code(code);

    Ok(device.create_shader_module(&info, None)?)
}
