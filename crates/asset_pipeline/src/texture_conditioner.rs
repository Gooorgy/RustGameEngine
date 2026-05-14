use assets::{load_image, write_etex};
use std::fmt;
use std::path::Path;

#[derive(Debug)]
pub enum TextureConditionError {
    Io(std::io::Error),
    UnsupportedFormat(String),
    Image(image::ImageError),
    Write(assets::EtexError),
}

impl fmt::Display for TextureConditionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TextureConditionError::Io(e) => write!(f, "io: {}", e),
            TextureConditionError::UnsupportedFormat(e) => {
                write!(f, "unsupported texture format '.{}'", e)
            }
            TextureConditionError::Image(e) => write!(f, "image: {}", e),
            TextureConditionError::Write(e) => write!(f, "write: {}", e),
        }
    }
}

impl From<std::io::Error> for TextureConditionError {
    fn from(e: std::io::Error) -> Self { TextureConditionError::Io(e) }
}
impl From<image::ImageError> for TextureConditionError {
    fn from(e: image::ImageError) -> Self { TextureConditionError::Image(e) }
}
impl From<assets::EtexError> for TextureConditionError {
    fn from(e: assets::EtexError) -> Self { TextureConditionError::Write(e) }
}

pub struct TextureConditioner;

impl TextureConditioner {
    /// Reads a source image (`.png`, `.jpg`, `.hdr`, etc.) and writes a cooked
    /// `.etex` binary to `dst_path`, creating parent directories as needed.
    /// All formats are converted to RGBA8 for now.
    pub fn condition(src_path: &Path, dst_path: &Path) -> Result<(), TextureConditionError> {
        match src_path.extension().and_then(|e| e.to_str()) {
            Some("png" | "jpg" | "jpeg" | "hdr" | "exr" | "bmp" | "tga") => {}
            Some(ext) => return Err(TextureConditionError::UnsupportedFormat(ext.to_string())),
            None => return Err(TextureConditionError::UnsupportedFormat("(none)".to_string())),
        }

        let image = load_image(src_path)?;
        if let Some(parent) = dst_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        write_etex(dst_path, &image.image_data, image.width, image.height)?;
        Ok(())
    }
}