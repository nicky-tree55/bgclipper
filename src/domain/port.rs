use crate::domain::color::Color;

/// RGBA image data with dimensions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImageData {
    /// Raw RGBA pixel buffer (4 bytes per pixel).
    pub pixels: Vec<u8>,
    /// Image width in pixels.
    pub width: u32,
    /// Image height in pixels.
    pub height: u32,
}

/// Port for reading and writing images on the system clipboard.
///
/// Implementations handle OS-specific clipboard access.
/// The domain layer depends only on this trait, not on concrete implementations.
pub trait ClipboardPort {
    /// The error type returned by clipboard operations.
    type Error: std::error::Error;

    /// Reads an image from the clipboard as RGBA pixel data.
    ///
    /// Returns `Ok(Some(ImageData))` if an image is available,
    /// `Ok(None)` if the clipboard does not contain an image,
    /// or `Err` if an OS-level error occurs.
    fn get_image(&self) -> Result<Option<ImageData>, Self::Error>;

    /// Writes RGBA pixel data to the clipboard as an image.
    fn set_image(&self, image: &ImageData) -> Result<(), Self::Error>;
}

/// Port for reading and writing application configuration.
///
/// Implementations handle config file I/O (e.g., TOML parsing).
/// The domain layer depends only on this trait.
pub trait ConfigPort {
    /// The error type returned by config operations.
    type Error: std::error::Error;

    /// Loads the target color from the configuration.
    ///
    /// Returns the configured target color, or a default if no config exists.
    fn load_target_color(&self) -> Result<Color, Self::Error>;

    /// Saves the target color to the configuration.
    fn save_target_color(&self, color: &Color) -> Result<(), Self::Error>;

    /// Ensures the config file exists.
    ///
    /// If the config file does not exist, creates it with default settings.
    /// If the file already exists, does nothing.
    fn ensure_config_exists(&self) -> Result<(), Self::Error>;
}
