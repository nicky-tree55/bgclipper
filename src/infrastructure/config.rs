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
    /// Global hotkey string (e.g. "CmdOrCtrl+Shift+T").
    /// Parsed by the `global-hotkey` crate's accelerator parser.
    hotkey: Option<String>,
}

/// RGB color section in the TOML config file.
#[derive(Debug, Serialize, Deserialize)]
struct ColorConfig {
    r: u8,
    g: u8,
    b: u8,
}

/// Default hotkey accelerator string.
const DEFAULT_HOTKEY: &str = "CmdOrCtrl+Shift+T";

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

    /// Returns the raw hotkey string from the config file, or the default.
    fn load_hotkey_raw(&self) -> String {
        let content = match fs::read_to_string(&self.path) {
            Ok(c) => c,
            Err(_) => return DEFAULT_HOTKEY.to_string(),
        };

        let config: ConfigFile = match toml::from_str(&content) {
            Ok(c) => c,
            Err(_) => return DEFAULT_HOTKEY.to_string(),
        };

        config
            .hotkey
            .unwrap_or_else(|| DEFAULT_HOTKEY.to_string())
    }

    /// Loads the hotkey accelerator string from the config file.
    ///
    /// If the config file is missing, unreadable, or the hotkey field is absent,
    /// returns the default hotkey (`CmdOrCtrl+Shift+T`).
    pub fn load_hotkey(&self) -> String {
        let raw = self.load_hotkey_raw();

        // Validate by attempting to parse; fall back to default on failure
        use global_hotkey::hotkey::HotKey;
        match raw.parse::<HotKey>() {
            Ok(_) => {
                debug!("hotkey loaded: {raw}");
                raw
            }
            Err(e) => {
                log::warn!(
                    "invalid hotkey {raw:?} in config ({e}), using default: {DEFAULT_HOTKEY}"
                );
                DEFAULT_HOTKEY.to_string()
            }
        }
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
        let existing_hotkey = self.load_hotkey_raw();

        let config = ConfigFile {
            target_color: ColorConfig {
                r: color.r(),
                g: color.g(),
                b: color.b(),
            },
            hotkey: Some(existing_hotkey),
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
            let config = ConfigFile {
                target_color: ColorConfig {
                    r: 255,
                    g: 255,
                    b: 255,
                },
                hotkey: Some(DEFAULT_HOTKEY.to_string()),
            };
            let content = toml::to_string(&config)?;
            if let Some(parent) = self.path.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::write(&self.path, content)?;
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

    #[test]
    fn load_hotkey_returns_default_when_file_missing() {
        let (provider, _dir) = temp_provider();
        let hotkey = provider.load_hotkey();
        assert_eq!(hotkey, "CmdOrCtrl+Shift+T");
    }

    #[test]
    fn load_hotkey_returns_value_from_config() {
        let (provider, _dir) = temp_provider();
        let content = r#"
hotkey = "CmdOrCtrl+Shift+B"

[target_color]
r = 255
g = 255
b = 255
"#;
        fs::write(&provider.path, content).unwrap();
        let hotkey = provider.load_hotkey();
        assert_eq!(hotkey, "CmdOrCtrl+Shift+B");
    }

    #[test]
    fn load_hotkey_falls_back_on_invalid_value() {
        let (provider, _dir) = temp_provider();
        let content = r#"
hotkey = "not-a-valid-hotkey!!!"

[target_color]
r = 255
g = 255
b = 255
"#;
        fs::write(&provider.path, content).unwrap();
        let hotkey = provider.load_hotkey();
        assert_eq!(hotkey, "CmdOrCtrl+Shift+T");
    }

    #[test]
    fn load_hotkey_falls_back_when_field_absent() {
        let (provider, _dir) = temp_provider();
        let content = r#"
[target_color]
r = 255
g = 255
b = 255
"#;
        fs::write(&provider.path, content).unwrap();
        let hotkey = provider.load_hotkey();
        assert_eq!(hotkey, "CmdOrCtrl+Shift+T");
    }

    #[test]
    fn ensure_config_includes_hotkey() {
        let (provider, _dir) = temp_provider();
        provider.ensure_config_exists().unwrap();
        let content = fs::read_to_string(&provider.path).unwrap();
        assert!(content.contains("hotkey"));
        assert!(content.contains("CmdOrCtrl+Shift+T"));
    }

    #[test]
    fn save_target_color_preserves_hotkey() {
        let (provider, _dir) = temp_provider();
        let content = r#"
hotkey = "CmdOrCtrl+Shift+X"

[target_color]
r = 255
g = 255
b = 255
"#;
        fs::write(&provider.path, content).unwrap();

        provider.save_target_color(&Color::new(10, 20, 30)).unwrap();

        let reloaded = fs::read_to_string(&provider.path).unwrap();
        assert!(reloaded.contains("CmdOrCtrl+Shift+X"));
        let loaded = provider.load_target_color().unwrap();
        assert_eq!(loaded, Color::new(10, 20, 30));
    }
}
