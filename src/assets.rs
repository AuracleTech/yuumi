use crate::{
    app::AppData,
    model,
    texture::{self, Texture},
};
use anyhow::{anyhow, Result};
use cgmath::Point3;
use std::collections::HashMap;
use vulkanalia::prelude::v1_0::*;

#[derive(Debug)]
pub(crate) struct Assets {
    pub(crate) meshes: HashMap<String, Mesh>,
    pub(crate) active_mesh: Vec<String>,
    pub(crate) textures: HashMap<String, Texture>,
}
impl Default for Assets {
    fn default() -> Self {
        Self {
            meshes: HashMap::new(),
            active_mesh: Vec::new(),
            textures: HashMap::new(),
        }
    }
}
impl Assets {
    pub(crate) fn load_model(
        &mut self,
        name: &str,
        instance: &mut Instance,
        device: &mut Device,
        data: &mut AppData,
    ) -> Result<()> {
        if self.meshes.contains_key(name) {
            return Err(anyhow!("Mesh name already in use: {}", name));
        }

        self.meshes.insert(
            name.to_string(),
            model::load_model(name, instance, device, data)?,
        );
        Ok(())
    }

    pub(crate) fn load_texture(
        &mut self,
        name: &str,
        instance: &mut Instance,
        device: &mut Device,
        data: &mut AppData,
    ) -> Result<()> {
        if self.textures.contains_key(name) {
            return Err(anyhow!("Mesh name already in use: {}", name));
        }

        self.textures.insert(
            name.to_string(),
            texture::load_texture(name, instance, device, data)?,
        );
        Ok(())
    }
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
