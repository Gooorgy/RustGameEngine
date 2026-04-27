use assets::write_emesh;
use nalgebra::{Vector2, Vector3};
use rendering_backend::vertex::Vertex;
use std::fmt;
use std::path::Path;

#[derive(Debug)]
pub enum MeshConditionError {
    Io(std::io::Error),
    UnsupportedFormat(String),
    Obj(tobj::LoadError),
    Gltf(gltf::Error),
    NoMesh,
    NoPrimitive,
    NoPositions,
    NoIndices,
    Write(assets::EmeshError),
}

impl fmt::Display for MeshConditionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MeshConditionError::Io(e) => write!(f, "io: {}", e),
            MeshConditionError::UnsupportedFormat(e) => write!(f, "unsupported mesh format '.{}'", e),
            MeshConditionError::Obj(e) => write!(f, "obj: {}", e),
            MeshConditionError::Gltf(e) => write!(f, "gltf: {}", e),
            MeshConditionError::NoMesh => write!(f, "no mesh found in file"),
            MeshConditionError::NoPrimitive => write!(f, "no primitive found in mesh"),
            MeshConditionError::NoPositions => write!(f, "mesh has no POSITION attribute"),
            MeshConditionError::NoIndices => write!(f, "mesh has no indices"),
            MeshConditionError::Write(e) => write!(f, "write: {}", e),
        }
    }
}

impl From<std::io::Error> for MeshConditionError {
    fn from(e: std::io::Error) -> Self { MeshConditionError::Io(e) }
}
impl From<tobj::LoadError> for MeshConditionError {
    fn from(e: tobj::LoadError) -> Self { MeshConditionError::Obj(e) }
}
impl From<gltf::Error> for MeshConditionError {
    fn from(e: gltf::Error) -> Self { MeshConditionError::Gltf(e) }
}
impl From<assets::EmeshError> for MeshConditionError {
    fn from(e: assets::EmeshError) -> Self { MeshConditionError::Write(e) }
}

pub struct MeshConditioner;

impl MeshConditioner {
    /// Reads a source mesh (`.obj`, `.gltf`, `.glb`) and writes a cooked
    /// `.emesh` binary to `dst_path`, creating parent directories as needed.
    pub fn condition(src_path: &Path, dst_path: &Path) -> Result<(), MeshConditionError> {
        let (vertices, indices) = match src_path.extension().and_then(|e| e.to_str()) {
            Some("obj") => Self::load_obj(src_path)?,
            Some("gltf") | Some("glb") => Self::load_gltf(src_path)?,
            Some(ext) => return Err(MeshConditionError::UnsupportedFormat(ext.to_string())),
            None => return Err(MeshConditionError::UnsupportedFormat("(none)".to_string())),
        };

        if let Some(parent) = dst_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        write_emesh(dst_path, &vertices, &indices)?;
        Ok(())
    }

    fn load_obj(path: &Path) -> Result<(Vec<Vertex>, Vec<u32>), MeshConditionError> {
        let (models, _) = tobj::load_obj(path, &tobj::GPU_LOAD_OPTIONS)?;
        let model = models.into_iter().next().ok_or(MeshConditionError::NoMesh)?;
        let mesh = &model.mesh;

        let vert_count = mesh.positions.len() / 3;
        let mut vertices = Vec::with_capacity(vert_count);

        for i in 0..vert_count {
            let pos = Vector3::new(
                mesh.positions[i * 3],
                mesh.positions[i * 3 + 1],
                mesh.positions[i * 3 + 2],
            );
            let normal = if mesh.normals.len() >= (i + 1) * 3 {
                Vector3::new(
                    mesh.normals[i * 3],
                    mesh.normals[i * 3 + 1],
                    mesh.normals[i * 3 + 2],
                )
            } else {
                Vector3::new(0.0, 1.0, 0.0)
            };
            let tex_coord = if mesh.texcoords.len() >= (i + 1) * 2 {
                Vector2::new(mesh.texcoords[i * 2], mesh.texcoords[i * 2 + 1])
            } else {
                Vector2::new(0.0, 0.0)
            };

            vertices.push(Vertex {
                pos,
                color: Vector3::new(1.0, 1.0, 1.0),
                tex_coord,
                normal,
                ..Default::default()
            });
        }

        Ok((vertices, mesh.indices.clone()))
    }

    fn load_gltf(path: &Path) -> Result<(Vec<Vertex>, Vec<u32>), MeshConditionError> {
        let (document, buffers, _) = gltf::import(path)?;

        let mesh = document.meshes().next().ok_or(MeshConditionError::NoMesh)?;
        let primitive = mesh.primitives().next().ok_or(MeshConditionError::NoPrimitive)?;
        let reader = primitive.reader(|buf| Some(&buffers[buf.index()]));

        let positions: Vec<[f32; 3]> = reader
            .read_positions()
            .ok_or(MeshConditionError::NoPositions)?
            .collect();

        let normals: Vec<[f32; 3]> = reader
            .read_normals()
            .map(|iter| iter.collect())
            .unwrap_or_else(|| vec![[0.0, 1.0, 0.0]; positions.len()]);

        let tex_coords: Vec<[f32; 2]> = reader
            .read_tex_coords(0)
            .map(|iter| iter.into_f32().collect())
            .unwrap_or_else(|| vec![[0.0, 0.0]; positions.len()]);

        let indices: Vec<u32> = reader
            .read_indices()
            .ok_or(MeshConditionError::NoIndices)?
            .into_u32()
            .collect();

        let vertices = positions
            .iter()
            .zip(normals.iter())
            .zip(tex_coords.iter())
            .map(|((pos, norm), uv)| Vertex {
                pos: Vector3::new(pos[0], pos[1], pos[2]),
                normal: Vector3::new(norm[0], norm[1], norm[2]),
                tex_coord: Vector2::new(uv[0], uv[1]),
                color: Vector3::new(1.0, 1.0, 1.0),
                ..Default::default()
            })
            .collect();

        Ok((vertices, indices))
    }
}