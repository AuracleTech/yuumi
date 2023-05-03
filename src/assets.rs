use crate::{
    app::AppData,
    camera::Camera,
    model::{self, Model},
    texture::{self, Texture},
};
use anyhow::{anyhow, Result};
use std::collections::HashMap;
use vulkanalia::prelude::v1_0::*;

#[derive(Debug)]
pub struct Assets {
    pub cameras: HashMap<String, Camera>,
    pub(crate) active_camera: String,
    pub(crate) models: HashMap<String, Model>,
    pub(crate) active_models: Vec<String>,
    pub(crate) textures: HashMap<String, Texture>,
}
impl Default for Assets {
    fn default() -> Self {
        Self {
            cameras: HashMap::new(),
            active_camera: String::new(),
            models: HashMap::new(),
            active_models: Vec::new(),
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
        if self.models.contains_key(name) {
            return Err(anyhow!("Mesh name already in use: {}", name));
        }

        self.models.insert(
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
