use anyhow::Result;

use crate::{app::AppData, vk_vertex::Vertex};

pub(crate) fn load_model(data: &mut AppData) -> Result<()> {
    let mut reader = std::io::BufReader::new(std::fs::File::open("assets/models/viking_room.obj")?);

    let (models, _) = tobj::load_obj_buf(
        &mut reader,
        &tobj::LoadOptions {
            triangulate: true,
            ..Default::default()
        },
        |_| Ok(Default::default()),
    )?;

    for model in &models {
        for index in &model.mesh.indices {
            let pos_offset = (3 * index) as usize;
            let tex_coord_offset = (2 * index) as usize;

            let vertex = Vertex {
                pos: cgmath::vec3(
                    model.mesh.positions[pos_offset],
                    model.mesh.positions[pos_offset + 1],
                    model.mesh.positions[pos_offset + 2],
                ),
                color: cgmath::vec3(1.0, 1.0, 1.0),
                tex_coord: cgmath::vec2(
                    model.mesh.texcoords[tex_coord_offset],
                    1.0 - model.mesh.texcoords[tex_coord_offset + 1],
                ),
            };

            data.vertices.push(vertex);
            data.indices.push(data.indices.len() as u32); // TODO: Fix this.
        }
    }

    Ok(())
}
