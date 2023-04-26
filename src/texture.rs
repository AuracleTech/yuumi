use crate::app::AppData;
use crate::image_view::create_image_view;
use crate::texture_image::create_texture_image;
use crate::texture_sampler::create_texture_sampler;
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;
use vulkanalia::prelude::v1_0::*;
use vulkanalia::{Device, Instance};

#[derive(Debug)]
pub(crate) struct Texture {
    pub(crate) image: vk::Image,
    pub(crate) image_view: vk::ImageView,
    pub(crate) image_memory: vk::DeviceMemory,
    pub(crate) _width: u32,
    pub(crate) _height: u32,
    pub(crate) _mip_levels: u32,
    // OPTIMIZE use a reference to the image view to reuse the same image view for multiple textures
    pub(crate) _format: vk::Format,
    // OPTIMIZE use a reference to the texture sampler to reuse the same sampler for multiple textures
    pub(crate) sampler: vk::Sampler,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct SerializedTexture {
    pub(crate) width: u32,
    pub(crate) height: u32,
    pub(crate) pixels: Vec<u8>,
}

pub(crate) fn load_texture(
    name: &str,
    instance: &mut Instance,
    device: &mut Device,
    data: &mut AppData,
) -> Result<Texture> {
    let supported_extensions = vec!["bin", "png"];

    let extension = supported_extensions
        .iter()
        .find_map(|extension| {
            let path = format!("assets/textures/{}.{}", name, extension);
            if Path::new(&path).exists() {
                Some(extension)
            } else {
                None
            }
        })
        .ok_or(anyhow!("no supported texture found"))?;

    if *extension != "bin" {
        let path = format!("assets/textures/{}.{}", name, extension);
        let (pixels, width, height) = match extension.as_ref() {
            "png" => load_suboptimal_png(&path)?,
            _ => Err(anyhow!("unsupported file extension: {}", extension))?,
        };
        save_optimal(&name, pixels, width, height)?;
    }

    let path = format!("assets/textures/{}.bin", name);
    let (pixels, width, height) = load_optimal(&path)?;
    let size = pixels.len() as u64;

    let (image, image_memory, mip_levels) =
        unsafe { create_texture_image(instance, device, data, &pixels, size, width, height)? };

    // OPTIMIZE reuse image view and sampler for the most textures possible
    let format = vk::Format::R8G8B8A8_SRGB;
    let aspects = vk::ImageAspectFlags::COLOR;
    let image_view = unsafe { create_image_view(device, &image, &format, &aspects, &mip_levels)? };
    let sampler = unsafe { create_texture_sampler(device, data, &mip_levels)? };

    Ok(Texture {
        image,
        image_view,
        image_memory,
        _mip_levels: mip_levels,
        _width: width,
        _height: height,
        _format: vk::Format::R8G8B8A8_SRGB,
        sampler,
    })
}

fn load_suboptimal_png(path: &str) -> Result<(Vec<u8>, u32, u32)> {
    let image = std::fs::File::open(path)?;

    let decoder = png::Decoder::new(image);
    let mut reader = decoder.read_info()?;

    let mut pixels = vec![0; reader.info().raw_bytes()];
    reader.next_frame(&mut pixels)?;

    let (width, height) = reader.info().size();

    Ok((pixels, width, height))
}

fn load_optimal(path: &str) -> Result<(Vec<u8>, u32, u32)> {
    let mut reader = std::io::BufReader::new(std::fs::File::open(path)?);
    let serialized: SerializedTexture = bincode::deserialize_from(&mut reader)?;
    Ok((serialized.pixels, serialized.width, serialized.height))
}

fn save_optimal(name: &str, pixels: Vec<u8>, width: u32, height: u32) -> Result<()> {
    let path = format!("assets/textures/{}.bin", name);
    let mut writer = std::io::BufWriter::new(std::fs::File::create(path)?);
    bincode::serialize_into(
        &mut writer,
        &SerializedTexture {
            width,
            height,
            pixels,
        },
    )?;
    Ok(())
}
