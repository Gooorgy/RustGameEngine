use std::fmt;
use std::path::Path;
use common::ImageData;

const MAGIC: [u8; 4] = *b"ETEX";
const VERSION: u32 = 1;

#[derive(Debug)]
pub enum EtexError {
    Io(std::io::Error),
    InvalidMagic,
    UnsupportedVersion(u32),
    Truncated,
}

impl fmt::Display for EtexError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EtexError::Io(e) => write!(f, "io: {}", e),
            EtexError::InvalidMagic => write!(f, "invalid .etex magic bytes"),
            EtexError::UnsupportedVersion(v) => write!(f, "unsupported .etex version {}", v),
            EtexError::Truncated => write!(f, ".etex file is truncated"),
        }
    }
}

/// Writes RGBA8 pixel data to a `.etex` binary file.
///
/// Format: 4-byte magic + version u32 + width u32 + height u32 + raw RGBA8 bytes
/// (all little-endian).
pub fn write_etex(path: &Path, image_data: &ImageData) -> Result<(), EtexError> {
    let mut buf = Vec::with_capacity(16 + image_data.pixels.len());
    buf.extend_from_slice(&MAGIC);
    buf.extend_from_slice(&VERSION.to_le_bytes());
    buf.extend_from_slice(&image_data.width.to_le_bytes());
    buf.extend_from_slice(&image_data.height.to_le_bytes());
    buf.extend_from_slice(&image_data.pixels);
    std::fs::write(path, buf).map_err(EtexError::Io)
}

/// Reads a `.etex` binary file and returns a ready-to-use `ImageAsset`.
pub fn read_etex(path: &Path) -> Result<ImageData, EtexError> {
    let data = std::fs::read(path).map_err(EtexError::Io)?;

    if data.len() < 16 {
        return Err(EtexError::Truncated);
    }
    if data[0..4] != MAGIC {
        return Err(EtexError::InvalidMagic);
    }
    let version = u32::from_le_bytes(data[4..8].try_into().unwrap());
    if version != VERSION {
        return Err(EtexError::UnsupportedVersion(version));
    }
    let width = u32::from_le_bytes(data[8..12].try_into().unwrap());
    let height = u32::from_le_bytes(data[12..16].try_into().unwrap());

    let expected = (width * height * 4) as usize;
    if data.len() < 16 + expected {
        return Err(EtexError::Truncated);
    }

    Ok(ImageData {
        pixels: data[16..16 + expected].to_vec(),
        width,
        height,
    })
}