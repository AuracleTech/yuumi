use std::collections::HashMap;

#[derive(Debug)]
pub(crate) struct Assets {
    textures: HashMap<String, Texture>,
    meshes: HashMap<String, Mesh>,
    // Add more asset types here as needed
}

impl Default for Assets {
    fn default() -> Self {
        Self {
            textures: HashMap::new(),
            meshes: HashMap::new(),
        }
    }
}

#[derive(Debug)]
pub(crate) struct Texture {
    // Texture data here
}

#[derive(Debug)]
pub(crate) struct Mesh {
    // Mesh data here
}
