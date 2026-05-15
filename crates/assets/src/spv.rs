use common::ShaderData;
use std::fmt;
use std::path::Path;

const MAGIC: u32 = 0x07230203;
const HEADER_WORDS: usize = 5;

#[derive(Debug)]
pub enum SpvError {
    Io(std::io::Error),
    /// File is smaller than the 20-byte SPIR-V header.
    TooShort(usize),
    /// File size is not a multiple of 4 bytes.
    UnalignedSize(usize),
    /// First word does not match the SPIR-V magic number.
    InvalidMagic(u32),
}

impl fmt::Display for SpvError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SpvError::Io(e) => write!(f, "io: {}", e),
            SpvError::TooShort(n) => {
                write!(f, "file too short ({} bytes, need at least 20)", n)
            }
            SpvError::UnalignedSize(n) => {
                write!(f, "file size {} is not a multiple of 4", n)
            }
            SpvError::InvalidMagic(m) => {
                write!(f, "invalid magic {:#010x} (expected {:#010x})", m, MAGIC)
            }
        }
    }
}

impl From<std::io::Error> for SpvError {
    fn from(e: std::io::Error) -> Self { SpvError::Io(e) }
}

pub fn read_spv(path: &Path) -> Result<ShaderData, SpvError> {
    let bytes = std::fs::read(path)?;

    if bytes.len() < HEADER_WORDS * 4 {
        return Err(SpvError::TooShort(bytes.len()));
    }
    if bytes.len() % 4 != 0 {
        return Err(SpvError::UnalignedSize(bytes.len()));
    }

    let spv: Vec<u32> = bytes
        .chunks_exact(4)
        .map(|c| u32::from_le_bytes(c.try_into().unwrap()))
        .collect();

    if spv[0] != MAGIC {
        return Err(SpvError::InvalidMagic(spv[0]));
    }

    Ok(ShaderData { spv })
}