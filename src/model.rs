use crate::{
    app::AppData,
    instance_buffer::create_instance_buffer,
    mesh::{Mesh, SerializedMesh},
    vertex::{InstanceData, Vertex},
    vertex_buffer::{create_index_buffer, create_vertex_buffer},
};
use anyhow::{anyhow, Result};
use base64::Engine;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path::Path};
use vulkanalia::{Device, Instance};

#[derive(Debug)]
pub(crate) struct Model {
    pub(crate) meshes: Vec<Mesh>,
}

#[derive(Serialize, Deserialize)]
pub(crate) struct SerializedModel {
    pub(crate) meshes: Vec<SerializedMesh>,
}

pub(crate) fn load_model(
    name: &str,
    instance: &mut Instance,
    device: &mut Device,
    data: &mut AppData,
) -> Result<Model> {
    let supported_extensions = vec!["bin", "glb", "gltf"];

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
        let serialized = match extension.as_ref() {
            "gltf" => load_suboptimal_gltf(&path, &extension)?,
            "glb" => load_suboptimal_gltf(&path, &extension)?,
            _ => Err(anyhow!("unsupported file extension: {}", extension))?,
        };
        save_optimal(&name, serialized)?;
    }

    let path = format!("assets/models/{}.bin", name);
    let mut reader = std::io::BufReader::new(std::fs::File::open(path)?);
    let serialized: SerializedModel = bincode::deserialize_from(&mut reader)?;

    let mut model = Model { meshes: vec![] };

    for mesh in serialized.meshes {
        // FIX starts without instances
        let instance_data = InstanceData {
            model_matrix: vec![
                cgmath::Matrix4::from_translation(cgmath::Vector3::new(0.0, -1.25, 1.0)),
                cgmath::Matrix4::from_translation(cgmath::Vector3::new(0.0, 1.25, 1.0)),
                cgmath::Matrix4::from_translation(cgmath::Vector3::new(0.0, -1.25, -1.0)),
                cgmath::Matrix4::from_translation(cgmath::Vector3::new(0.0, 1.25, -1.0)),
            ],
        };

        let (vertex_buffer, vertex_buffer_memory) =
            unsafe { create_vertex_buffer(&mesh.vertices, instance, device, data)? };
        let (index_buffer, index_buffer_memory) =
            unsafe { create_index_buffer(&mesh.indices, instance, device, data)? };
        let (instance_buffer, instance_buffer_memory) =
            unsafe { create_instance_buffer(&instance_data, instance, device, data)? };

        model.meshes.push(Mesh {
            vertex_buffer,
            vertex_buffer_memory,
            index_count: mesh.indices.len() as u32,
            index_buffer,
            index_buffer_memory,
            instance_count: instance_data.model_matrix.len() as u32,
            instance_buffer,
            instance_buffer_memory,
        });
    }

    Ok(model)
}

fn save_optimal(name: &str, serialized: SerializedModel) -> Result<()> {
    let path = format!("assets/models/{}.bin", name);
    let mut writer = std::io::BufWriter::new(std::fs::File::create(path)?);

    let mut new_serialized = SerializedModel { meshes: vec![] };

    for mesh in &serialized.meshes {
        let mut unique_vertices: HashMap<Vertex, usize> = HashMap::new();
        let mut vertices = Vec::new();
        let mut indices = Vec::new();
        for index in &mesh.indices {
            let vertex = mesh.vertices[*index as usize];
            if let Some(index) = unique_vertices.get(&vertex) {
                indices.push(*index as u32);
            } else {
                let index = vertices.len();
                unique_vertices.insert(vertex, index);
                vertices.push(vertex);
                indices.push(index as u32);
            }
        }

        new_serialized
            .meshes
            .push(SerializedMesh { vertices, indices });
    }

    bincode::serialize_into(&mut writer, &new_serialized)?;

    Ok(())
}

fn load_suboptimal_gltf(path: &str, extension: &str) -> Result<SerializedModel> {
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

    let mut serialized = SerializedModel { meshes: vec![] };

    for mesh in gltf.meshes() {
        for primitive in mesh.primitives() {
            let mut vertices = Vec::new();
            let mut indices = Vec::new();

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

            serialized.meshes.push(SerializedMesh { vertices, indices });
        }
    }

    Ok(serialized)
}
