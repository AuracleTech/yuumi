use std::{collections::HashMap, path::Path};

use anyhow::{anyhow, Result};
use base64::Engine;
use vulkanalia::{Device, Instance};

use crate::{
    app::AppData,
    assets::Mesh,
    serializer::SerializedMesh,
    vertex::Vertex,
    vertex_buffer::{create_index_buffer, create_vertex_buffer},
};

pub(crate) struct Model {
    pub(crate) meshes: Vec<Mesh>,
}

pub(crate) fn load_model(
    name: &str,
    instance: &mut Instance,
    device: &mut Device,
    data: &mut AppData,
) -> Result<Mesh> {
    let supported_extensions = vec!["bin", "glb", "gltf", "obj"];

    let extension = supported_extensions
        .iter()
        .find_map(|extension| {
            let path = format!("assets/models/{}.{}", name, extension);
            if Path::new(&path).exists() {
                Some(extension)
            } else {
                None
            }
        })
        .ok_or(anyhow!("no supported model found"))?;

    if *extension != "bin" {
        let path = format!("assets/models/{}.{}", name, extension);
        let (vertices, indices) = match extension.as_ref() {
            "gltf" => load_suboptimal_gltf(&path, &extension)?,
            "glb" => load_suboptimal_gltf(&path, &extension)?,
            "obj" => load_suboptimal_obj(&path)?,
            _ => Err(anyhow!("unsupported extension"))?,
        };
        optimize_model(&name, &vertices, &indices)?;
    }

    let path = format!("assets/models/{}.bin", name);
    let (vertices, indices) = load_optimal(&path)?;

    // FIX starts without instances
    let instances_positions = vec![
        cgmath::Point3::new(0.0, -1.25, 1.0),
        cgmath::Point3::new(0.0, 1.25, 1.0),
        cgmath::Point3::new(0.0, -1.25, -1.0),
        cgmath::Point3::new(0.0, 1.25, -1.0),
    ];

    let (vertex_buffer, vertex_buffer_memory) =
        unsafe { create_vertex_buffer(&vertices, instance, device, data)? };
    let (index_buffer, index_buffer_memory) =
        unsafe { create_index_buffer(&indices, instance, device, data)? };

    Ok(Mesh {
        vertex_buffer,
        vertex_buffer_memory,
        index_buffer,
        index_buffer_memory,
        instances_positions,
        index_count: indices.len() as u32,
    })
}

fn load_suboptimal_obj(path: &str) -> Result<(Vec<Vertex>, Vec<u32>)> {
    let mut reader = std::io::BufReader::new(std::fs::File::open(path)?);

    let (models, _) = tobj::load_obj_buf(
        &mut reader,
        &tobj::LoadOptions {
            triangulate: true,
            ..Default::default()
        },
        |_| Ok(Default::default()),
    )?;

    let mut vertices = Vec::new();
    let mut indices = Vec::new();

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
            vertices.push(vertex);
            indices.push(indices.len() as u32);
        }
    }

    Ok((vertices, indices))
}

pub(crate) fn optimize_model(
    name: &str,
    old_vertices: &[Vertex],
    old_indices: &Vec<u32>,
) -> Result<()> {
    let path = format!("assets/models/{}.bin", name);
    let mut writer = std::io::BufWriter::new(std::fs::File::create(path)?);

    let mut unique_vertices: HashMap<Vertex, usize> = HashMap::new();
    let mut vertices = Vec::new();
    let mut indices = Vec::new();
    for index in old_indices {
        let vertex = old_vertices[*index as usize];
        if let Some(index) = unique_vertices.get(&vertex) {
            indices.push(*index as u32);
        } else {
            let index = vertices.len();
            unique_vertices.insert(vertex, index);
            vertices.push(vertex);
            indices.push(index as u32);
        }
    }

    bincode::serialize_into(&mut writer, &SerializedMesh { vertices, indices })?;

    Ok(())
}

pub(crate) fn load_optimal(path: &str) -> Result<(Vec<Vertex>, Vec<u32>)> {
    let mut reader = std::io::BufReader::new(std::fs::File::open(path)?);
    let serialized_mesh: SerializedMesh = bincode::deserialize_from(&mut reader)?;
    Ok((serialized_mesh.vertices, serialized_mesh.indices))
}

pub(crate) fn load_suboptimal_gltf(path: &str, extension: &str) -> Result<(Vec<Vertex>, Vec<u32>)> {
    let (gltf, buffers, _) = gltf::import(&path).expect("Failed to import gltf file");

    let mut buffer_data = Vec::new();
    for buffer in gltf.buffers() {
        match buffer.source() {
            gltf::buffer::Source::Bin => {
                buffer_data.push(buffers[buffer.index()].clone());
            }
            gltf::buffer::Source::Uri(uri) => {
                if extension != "glb" {
                    let uri = uri.trim_start_matches("data:application/octet-stream;base64,");
                    let data = uri.as_bytes();
                    let bin = base64::engine::general_purpose::STANDARD
                        .decode(data)
                        .expect("Failed to decode buffer data");
                    buffer_data.push(gltf::buffer::Data(bin));
                } else {
                    let bin = std::fs::read(uri).expect("Failed to read buffer data");
                    buffer_data.push(gltf::buffer::Data(bin));
                }
            }
        }
    }

    // let mut materials = Vec::new();
    // let mut material_meshes_pairs = HashMap::new();
    // for gltf_material in gltf.materials() {
    //     let pbr = gltf_material.pbr_metallic_roughness();
    //     if let Some(gltf_texture) = &pbr.base_color_texture().map(|tex| tex.texture()) {
    //         // println!(
    //         //     "Loading PBR for model {} with texture {}",
    //         //     path.display(),
    //         //     gltf_texture.index()
    //         // );

    //         let wrap_s = match gltf_texture.sampler().wrap_s() {
    //             WrappingMode::ClampToEdge => gl::CLAMP_TO_EDGE,
    //             WrappingMode::MirroredRepeat => gl::MIRRORED_REPEAT,
    //             WrappingMode::Repeat => gl::REPEAT,
    //         };
    //         let wrap_t = match gltf_texture.sampler().wrap_t() {
    //             WrappingMode::ClampToEdge => gl::CLAMP_TO_EDGE,
    //             WrappingMode::MirroredRepeat => gl::MIRRORED_REPEAT,
    //             WrappingMode::Repeat => gl::REPEAT,
    //         };
    //         if let Some(filter_min) = gltf_texture.sampler().min_filter() {
    //             match filter_min {
    //                 MinFilter::Nearest => gl::NEAREST,
    //                 MinFilter::Linear => gl::LINEAR,
    //                 MinFilter::NearestMipmapNearest => gl::NEAREST_MIPMAP_NEAREST,
    //                 MinFilter::LinearMipmapNearest => gl::LINEAR_MIPMAP_NEAREST,
    //                 MinFilter::NearestMipmapLinear => gl::NEAREST_MIPMAP_LINEAR,
    //                 MinFilter::LinearMipmapLinear => gl::LINEAR_MIPMAP_LINEAR,
    //             };
    //         }
    //         if let Some(filter_mag) = gltf_texture.sampler().mag_filter() {
    //             match filter_mag {
    //                 MagFilter::Nearest => gl::NEAREST,
    //                 MagFilter::Linear => gl::LINEAR,
    //             };
    //         }

    //         let texture_source = gltf_texture.source().source();

    //         let albedo_image = match texture_source {
    //             Source::Uri { uri, .. } => Image::from_uri(uri),
    //             Source::View { view, .. } => {
    //                 let data = &buffer_data[view.buffer().index()][view.offset()..];
    //                 Image::from_data(data)
    //             }
    //         };
    //         let mut albedo = Texture::new(albedo_image);
    //         albedo.gl_s_wrapping = wrap_s;
    //         albedo.gl_t_wrapping = wrap_t;
    //         // TODO add min and mag filter
    //         // TODO mipmaps? and all the other texture options

    //         materials.push(Material::Pbr { albedo });

    //         // TEMPORARY - ASSIGN EVERY MESH TO THE FIRST MATERIAL
    //         material_meshes_pairs.insert(0, vec![0]);
    //     }

    // if let Some(normal_texture) = gltf_material.normal_texture() {
    //     println!(
    //         "Loading normal texture for model {} with texture {}",
    //         path.display(),
    //         normal_texture.texture().index()
    //     );

    //     let wrap_s = match normal_texture.texture().sampler().wrap_s() {
    //         WrappingMode::ClampToEdge => gl::CLAMP_TO_EDGE,
    //         WrappingMode::MirroredRepeat => gl::MIRRORED_REPEAT,
    //         WrappingMode::Repeat => gl::REPEAT,
    //     };

    //     let wrap_t = match normal_texture.texture().sampler().wrap_t() {
    //         WrappingMode::ClampToEdge => gl::CLAMP_TO_EDGE,
    //         WrappingMode::MirroredRepeat => gl::MIRRORED_REPEAT,
    //         WrappingMode::Repeat => gl::REPEAT,
    //     };

    //     if let Some(filter_min) = normal_texture.texture().sampler().min_filter() {
    //         match filter_min {
    //             MinFilter::Nearest => gl::NEAREST,
    //             MinFilter::Linear => gl::LINEAR,
    //             MinFilter::NearestMipmapNearest => gl::NEAREST_MIPMAP_NEAREST,
    //             MinFilter::LinearMipmapNearest => gl::LINEAR_MIPMAP_NEAREST,
    //             MinFilter::NearestMipmapLinear => gl::NEAREST_MIPMAP_LINEAR,
    //             MinFilter::LinearMipmapLinear => gl::LINEAR_MIPMAP_LINEAR,
    //         };
    //     }

    //     if let Some(filter_mag) = normal_texture.texture().sampler().mag_filter() {
    //         match filter_mag {
    //             MagFilter::Nearest => gl::NEAREST,
    //             MagFilter::Linear => gl::LINEAR,
    //         };
    //     }

    //     let texture_source = normal_texture.texture().source().source();

    //     let normal_image = match texture_source {
    //         Source::Uri { uri, .. } => Image::from_uri(uri),
    //         Source::View { view, .. } => {
    //             let data = &buffer_data[view.buffer().index()][view.offset()..];
    //             Image::from_data(data)
    //         }
    //     };

    //     let mut normal = Texture::new(normal_image);
    //     normal.gl_s_wrapping = wrap_s;
    //     normal.gl_t_wrapping = wrap_t;
    //     // TODO add min and mag filter
    //     // TODO mipmaps? and all the other texture options

    //     materials.push(Material::Normal { normal });
    // }
    // }
    let mut vertices = Vec::new();
    let mut indices = Vec::new();

    for mesh in gltf.meshes() {
        for primitive in mesh.primitives() {
            let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));

            // TODO Vertex color - Needs a new type called VertexGroup or something check on blender
            if let Some(position_attribute) = reader.read_positions() {
                position_attribute.for_each(|position| {
                    vertices.push(Vertex {
                        pos: position.into(),
                        color: cgmath::vec3(0.0, 0.0, 0.0),
                        tex_coord: cgmath::vec2(0.0, 0.0),
                    })
                });
            }
            // if let Some(normal_attribute) = reader.read_normals() {
            //     let mut normal_index = 0;
            //     normal_attribute.for_each(|normal| {
            //         vertices[normal_index].normal = normal.into();
            //         normal_index += 1;
            //     });
            // }
            if let Some(tex_coord_attribute) = reader.read_tex_coords(0).map(|v| v.into_f32()) {
                let mut tex_coord_index = 0;
                tex_coord_attribute.for_each(|tex_coord| {
                    vertices[tex_coord_index].tex_coord = tex_coord.into();
                    tex_coord_index += 1;
                });
            }

            if let Some(indices_raw) = reader.read_indices() {
                indices.append(&mut indices_raw.into_u32().collect::<Vec<u32>>());
            }
        }
    }

    Ok((vertices, indices))
}
