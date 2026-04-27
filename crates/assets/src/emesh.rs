use rendering_backend::backend_impl::resource_manager::Mesh;
use rendering_backend::vertex::Vertex;
use std::fmt;
use std::path::Path;
use std::rc::Rc;

const MAGIC: [u8; 4] = *b"EMSH";
const VERSION: u32 = 1;

#[derive(Debug)]
pub enum EmeshError {
    Io(std::io::Error),
    InvalidMagic,
    UnsupportedVersion(u32),
    Truncated,
}

impl fmt::Display for EmeshError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EmeshError::Io(e) => write!(f, "io: {}", e),
            EmeshError::InvalidMagic => write!(f, "invalid .emesh magic bytes"),
            EmeshError::UnsupportedVersion(v) => write!(f, "unsupported .emesh version {}", v),
            EmeshError::Truncated => write!(f, ".emesh file is truncated"),
        }
    }
}

/// Writes vertices and indices to a `.emesh` binary file.
///
/// Format: 4-byte magic + version u32 + vertex_count u32 + index_count u32
/// + raw vertex bytes + raw index bytes (all little-endian).
pub fn write_emesh(path: &Path, vertices: &[Vertex], indices: &[u32]) -> Result<(), EmeshError> {
    let vertex_size = std::mem::size_of::<Vertex>();
    let mut buf = Vec::with_capacity(16 + vertices.len() * vertex_size + indices.len() * 4);

    buf.extend_from_slice(&MAGIC);
    buf.extend_from_slice(&VERSION.to_le_bytes());
    buf.extend_from_slice(&(vertices.len() as u32).to_le_bytes());
    buf.extend_from_slice(&(indices.len() as u32).to_le_bytes());

    // Safe: Vertex is #[repr(C)] with no padding that would expose uninit bytes
    let vert_bytes = unsafe {
        std::slice::from_raw_parts(vertices.as_ptr() as *const u8, vertices.len() * vertex_size)
    };
    buf.extend_from_slice(vert_bytes);

    let idx_bytes = unsafe {
        std::slice::from_raw_parts(indices.as_ptr() as *const u8, indices.len() * 4)
    };
    buf.extend_from_slice(idx_bytes);

    std::fs::write(path, buf).map_err(EmeshError::Io)
}

/// Reads a `.emesh` binary file and returns a ready-to-use `Mesh`.
pub fn read_emesh(path: &Path) -> Result<Rc<Mesh>, EmeshError> {
    let data = std::fs::read(path).map_err(EmeshError::Io)?;

    if data.len() < 16 {
        return Err(EmeshError::Truncated);
    }
    if data[0..4] != MAGIC {
        return Err(EmeshError::InvalidMagic);
    }
    let version = u32::from_le_bytes(data[4..8].try_into().unwrap());
    if version != VERSION {
        return Err(EmeshError::UnsupportedVersion(version));
    }
    let vertex_count = u32::from_le_bytes(data[8..12].try_into().unwrap()) as usize;
    let index_count = u32::from_le_bytes(data[12..16].try_into().unwrap()) as usize;

    let vertex_size = std::mem::size_of::<Vertex>();
    let vert_start = 16;
    let vert_end = vert_start + vertex_count * vertex_size;
    let idx_end = vert_end + index_count * 4;

    if data.len() < idx_end {
        return Err(EmeshError::Truncated);
    }

    // Safe: reading back data we wrote as Vertex, alignment guaranteed by Vec allocation
    let vertices = unsafe {
        let ptr = data[vert_start..vert_end].as_ptr() as *const Vertex;
        std::slice::from_raw_parts(ptr, vertex_count).to_vec()
    };
    let indices = unsafe {
        let ptr = data[vert_end..idx_end].as_ptr() as *const u32;
        std::slice::from_raw_parts(ptr, index_count).to_vec()
    };

    Ok(Rc::new(Mesh { vertices, indices }))
}