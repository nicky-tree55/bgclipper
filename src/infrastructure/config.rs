use std::fs;
use std::io;
use std::path::PathBuf;

use log::debug;
use serde::{Deserialize, Serialize};

use crate::domain::color::Color;
use crate::domain::port::ConfigPort;

/// Serializable configuration for the target color.
#[derive(Debug, Serialize, Deserialize)]
struct ConfigFile {
    target_color: ColorConfig,
}

/// RGB color section in the TOML config file.
#[derive(Debug, Serialize, Deserialize)]
struct ColorConfig {
    r: u8,
    g: u8,
    b: u8,
}

/// Errors that can occur during config file operations.
#[derive(Debug)]
pub enum ConfigError {
    /// Failed to read or write the config file.
    Io(io::Error),
    /// Failed to parse the TOML content.
    Parse(toml::de::Error),
    /// Failed to serialize the config to TOML.
    Serialize(toml::ser::Error),
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::Io(e) => write!(f, "config I/O error: {e}"),
            ConfigError::Parse(e) => write!(f, "config parse error: {e}"),
            ConfigError::Serialize(e) => write!(f, "config serialize error: {e}"),
        }
    }
}

impl std::error::Error for ConfigError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ConfigError::Io(e) => Some(e),
            ConfigError::Parse(e) => Some(e),
            ConfigError::Serialize(e) => Some(e),
        }
    }
}

impl From<io::Error> for ConfigError {
    fn from(e: io::Error) -> Self {
        ConfigError::Io(e)
    }
}

impl From<toml::de::Error> for ConfigError {
    fn from(e: toml::de::Error) -> Self {
        ConfigError::Parse(e)
    }
}

impl From<toml::ser::Error> for ConfigError {
    fn from(e: toml::ser::Error) -> Self {
        ConfigError::Serialize(e)
    }
}

/// TOML-based configuration provider.
///
/// Reads and writes the target color from a TOML config file.
/// The config file path is platform-dependent:
/// - macOS: `~/.config/bgclipper/config.toml`
/// - Windows: `%APPDATA%\bgclipper\config.toml`
#[derive(Debug)]
pub struct TomlConfigProvider {
    path: PathBuf,
}

impl TomlConfigProvider {
    /// Creates a provider using the platform-default config path.
    ///
    /// Returns `None` if the platform config directory cannot be determined.
    pub fn new() -> Option<Self> {
        let config_dir = dirs::config_dir()?;
        Some(Self {
            path: config_dir.join("bgclipper").join("config.toml"),
        })
    }

    /// Creates a provider with an explicit config file path.
    ///
    /// Useful for testing with temporary directories.
    pub fn with_path(path: PathBuf) -> Self {
        Self { path }
    }
}

impl ConfigPort for TomlConfigProvider {
    type Error = ConfigError;

    fn load_target_color(&self) -> Result<Color, Self::Error> {
        let content = match fs::read_to_string(&self.path) {
            Ok(c) => c,
            Err(e) if e.kind() == io::ErrorKind::NotFound => {
                debug!("config file not found, using defaults: {:?}", self.path);
                return Ok(Color::default());
            }
            Err(e) => return Err(e.into()),
        };

        let config: ConfigFile = toml::from_str(&content)?;
        debug!(
            "config loaded from {:?}: RGB({}, {}, {})",
            self.path, config.target_color.r, config.target_color.g, config.target_color.b
        );
        Ok(Color::new(
            config.target_color.r,
            config.target_color.g,
            config.target_color.b,
        ))
    }

    fn save_target_color(&self, color: &Color) -> Result<(), Self::Error> {
        let config = ConfigFile {
            target_color: ColorConfig {
                r: color.r(),
                g: color.g(),
                b: color.b(),
            },
        };

        let content = toml::to_string(&config)?;

        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)?;
        }

        fs::write(&self.path, content)?;
        Ok(())
    }

    fn ensure_config_exists(&self) -> Result<(), Self::Error> {
        if !self.path.exists() {
            debug!("creating default config at {:?}", self.path);
            self.save_target_color(&Color::default())?;
        } else {
            debug!("config file already exists: {:?}", self.path);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    fn temp_provider() -> (TomlConfigProvider, tempfile::TempDir) {
        let dir = tempfile::tempdir().expect("failed to create temp dir");
        let path = dir.path().join("config.toml");
        (TomlConfigProvider::with_path(path), dir)
    }

    #[test]
    fn load_returns_default_when_file_missing() {
        let (provider, _dir) = temp_provider();
        let color = provider.load_target_color().unwrap();
        assert_eq!(color, Color::default());
    }

    #[test]
    fn save_and_load_roundtrip() {
        let (provider, _dir) = temp_provider();
        let color = Color::new(100, 150, 200);
        provider.save_target_color(&color).unwrap();
        let loaded = provider.load_target_color().unwrap();
        assert_eq!(loaded, color);
    }

    #[test]
    fn save_creates_parent_directories() {
        let dir = tempfile::tempdir().expect("failed to create temp dir");
        let path = dir.path().join("nested").join("deep").join("config.toml");
        let provider = TomlConfigProvider::with_path(path.clone());

        provider.save_target_color(&Color::default()).unwrap();

        assert!(path.exists());
    }

    #[test]
    fn load_returns_error_for_malformed_toml() {
        let (provider, _dir) = temp_provider();
        fs::write(&provider.path, "not valid toml [[[").unwrap();

        let result = provider.load_target_color();
        assert!(result.is_err());
    }

    #[test]
    fn saved_file_is_valid_toml() {
        let (provider, _dir) = temp_provider();
        let color = Color::new(10, 20, 30);
        provider.save_target_color(&color).unwrap();

        let content = fs::read_to_string(&provider.path).unwrap();
        let config: ConfigFile = toml::from_str(&content).unwrap();
        assert_eq!(config.target_color.r, 10);
        assert_eq!(config.target_color.g, 20);
        assert_eq!(config.target_color.b, 30);
    }

    #[test]
    fn overwrite_existing_config() {
        let (provider, _dir) = temp_provider();

        provider.save_target_color(&Color::new(0, 0, 0)).unwrap();
        provider
            .save_target_color(&Color::new(255, 128, 64))
            .unwrap();

        let loaded = provider.load_target_color().unwrap();
        assert_eq!(loaded, Color::new(255, 128, 64));
    }

    #[test]
    fn ensure_config_exists_creates_file_when_missing() {
        let (provider, _dir) = temp_provider();
        assert!(!provider.path.exists());

        provider.ensure_config_exists().unwrap();

        assert!(provider.path.exists());
        let loaded = provider.load_target_color().unwrap();
        assert_eq!(loaded, Color::default());
    }

    #[test]
    fn ensure_config_exists_does_not_overwrite() {
        let (provider, _dir) = temp_provider();
        let custom = Color::new(10, 20, 30);
        provider.save_target_color(&custom).unwrap();

        provider.ensure_config_exists().unwrap();

        let loaded = provider.load_target_color().unwrap();
        assert_eq!(loaded, custom);
    }
}
