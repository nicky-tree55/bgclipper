use std::borrow::Cow;

use arboard::Clipboard;

use crate::domain::port::{ClipboardPort, ImageData};

/// Errors that can occur during clipboard operations.
#[derive(Debug)]
pub enum ClipboardError {
    /// An error from the underlying clipboard library.
    Arboard(arboard::Error),
}

impl std::fmt::Display for ClipboardError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ClipboardError::Arboard(e) => write!(f, "clipboard error: {e}"),
        }
    }
}

impl std::error::Error for ClipboardError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ClipboardError::Arboard(e) => Some(e),
        }
    }
}

impl From<arboard::Error> for ClipboardError {
    fn from(e: arboard::Error) -> Self {
        ClipboardError::Arboard(e)
    }
}

/// Clipboard provider backed by the `arboard` crate.
///
/// Provides cross-platform clipboard image access for macOS and Windows.
#[derive(Debug)]
pub struct ArboardClipboardProvider;

impl ArboardClipboardProvider {
    /// Creates a new provider.
    pub fn new() -> Self {
        Self
    }
}

impl Default for ArboardClipboardProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl ClipboardPort for ArboardClipboardProvider {
    type Error = ClipboardError;

    fn get_image(&self) -> Result<Option<ImageData>, Self::Error> {
        let mut clipboard = Clipboard::new()?;
        match clipboard.get_image() {
            Ok(img) => Ok(Some(ImageData {
                pixels: img.bytes.into_owned(),
                width: img.width as u32,
                height: img.height as u32,
            })),
            Err(arboard::Error::ContentNotAvailable) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    fn set_image(&self, image: &ImageData) -> Result<(), Self::Error> {
        let mut clipboard = Clipboard::new()?;
        let img = arboard::ImageData {
            width: image.width as usize,
            height: image.height as usize,
            bytes: Cow::Borrowed(&image.pixels),
        };
        clipboard.set_image(img)?;
        Ok(())
    }
}
