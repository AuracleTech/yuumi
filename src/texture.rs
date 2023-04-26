use std::path::Path;

use anyhow::{anyhow, Result};
use vulkanalia::prelude::v1_0::*;
use vulkanalia::{Device, Instance};

use crate::app::AppData;
use crate::image_view::create_image_view;
use crate::texture_sampler::create_texture_sampler;
use crate::{assets::Texture, texture_image::create_texture_image};

pub(crate) fn load_texture(
    name: &str,
    instance: &mut Instance,
    device: &mut Device,
    data: &mut AppData,
) -> Result<Texture> {
    let supported_extensions = vec!["png"];

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
        .ok_or(anyhow!("no supported model found"))?;

    // TODO optimize if not bin
    // if *extension != "bin" {
    //     let path = format!("assets/models/{}.{}", name, extension);
    //     let (vertices, indices) = match extension.as_ref() {
    //         "gltf" => load_suboptimal_gltf(&path, &extension)?,
    //         "glb" => load_suboptimal_gltf(&path, &extension)?,
    //         _ => Err(anyhow!("unsupported extension"))?,
    //     };
    //     optimize_model(&name, &vertices, &indices)?;
    // }

    // TEMP load png
    Ok(load_texture_png(name, instance, device, data)?)
}

fn load_texture_png(
    name: &str,
    instance: &mut Instance,
    device: &mut Device,
    data: &mut AppData,
) -> Result<Texture> {
    let path = format!("assets/textures/{}.png", name);
    let image = std::fs::File::open(path)?;

    let decoder = png::Decoder::new(image);
    let mut reader = decoder.read_info()?;

    let mut pixels = vec![0; reader.info().raw_bytes()];
    reader.next_frame(&mut pixels)?;

    let size = reader.info().raw_bytes() as u64;
    let (width, height) = reader.info().size();

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
