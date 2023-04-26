use serde::{Deserialize, Serialize};

use crate::vertex::Vertex;

#[derive(Serialize, Deserialize)]
pub(crate) struct SerializedMesh {
    pub(crate) vertices: Vec<Vertex>,
    pub(crate) indices: Vec<u32>,
}
