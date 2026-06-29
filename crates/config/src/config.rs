use serde::{Deserialize, Serialize};
use std::fmt;
use std::path::{Path, PathBuf};

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct ConfigFile {
    #[serde(default)]
    pub graphics_settings: GraphicsSettings,
    #[serde(default)]
    pub key_bindings: KeyBindings,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct GraphicsSettings {
    #[serde(default)]
    pub window_mode: WindowMode,
    #[serde(default)]
    pub resolution_settings: WindowResolution,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WindowResolution {
    pub width: u32,
    pub height: u32,
}

impl Default for WindowResolution {
    fn default() -> Self {
        Self {
            width: 1280,
            height: 720,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Default, Clone, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum WindowMode {
    Fullscreen,
    BorderlessFullscreen,
    #[default]
    Windowed,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct KeyBindings(pub std::collections::HashMap<String, String>);

#[derive(Debug)]
pub enum ConfigError {
    Io(std::io::Error),
    Parse(toml::de::Error),
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigError::Io(e) => write!(f, "io: {}", e),
            ConfigError::Parse(e) => write!(f, "parse: {}", e),
        }
    }
}

impl From<std::io::Error> for ConfigError {
    fn from(e: std::io::Error) -> Self {
        ConfigError::Io(e)
    }
}

impl From<toml::de::Error> for ConfigError {
    fn from(e: toml::de::Error) -> Self {
        ConfigError::Parse(e)
    }
}

impl ConfigFile {
    /// Loads the config from an explicit file path.
    pub fn load(path: &Path) -> Result<Self, ConfigError> {
        let content = std::fs::read_to_string(path)?;
        Ok(toml::from_str(&content)?)
    }

    /// Returns the OS-standard config directory for the given application name.
    ///
    /// - Windows: `%APPDATA%\<name>\`
    /// - Linux:   `$XDG_CONFIG_HOME/<name>/` (falls back to `~/.config/<name>/`)
    /// - macOS:   `~/Library/Application Support/<name>/`
    pub fn config_dir(app_name: &str) -> Option<PathBuf> {
        directories::ProjectDirs::from("", "", app_name).map(|d| d.config_dir().to_path_buf())
    }

    /// Loads config from the OS config directory for `app_name`. Returns
    /// `Default` silently if the file is absent or cannot be parsed.
    pub fn load_or_default(app_name: &str) -> Self {
        Self::config_dir(app_name)
            .map(|d| d.join("config.toml"))
            .and_then(|p| Self::load(&p).ok())
            .unwrap_or_default()
    }

    /// Saves the config to the OS config directory, creating it if needed.
    pub fn save(&self, app_name: &str) -> Result<(), ConfigError> {
        let dir = Self::config_dir(app_name).ok_or_else(|| {
            ConfigError::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "could not resolve OS config directory",
            ))
        })?;
        std::fs::create_dir_all(&dir)?;
        let content = toml::to_string_pretty(self).map_err(|e| {
            ConfigError::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                e.to_string(),
            ))
        })?;
        std::fs::write(dir.join("config.toml"), content)?;
        Ok(())
    }
}
