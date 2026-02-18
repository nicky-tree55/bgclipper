use log::debug;

use crate::domain::image_processor::make_transparent;
use crate::domain::port::{ClipboardPort, ConfigPort};

/// Result of processing a clipboard image.
#[derive(Debug, PartialEq, Eq)]
pub enum ProcessResult {
    /// An image was found and processed successfully.
    Processed,
    /// No image was found on the clipboard.
    NoImage,
}

/// Orchestrates the clipboard-to-transparent-image workflow.
///
/// Reads an image from the clipboard, applies transparency conversion
/// for the configured target color, and writes the result back.
///
/// Depends on port traits only — no concrete infrastructure references.
#[derive(Debug)]
pub struct ClipboardService<C, G>
where
    C: ClipboardPort,
    G: ConfigPort,
{
    clipboard: C,
    config: G,
}

impl<C, G> ClipboardService<C, G>
where
    C: ClipboardPort,
    G: ConfigPort,
{
    /// Creates a new service with the given clipboard and config providers.
    pub fn new(clipboard: C, config: G) -> Self {
        Self { clipboard, config }
    }

    /// Processes the current clipboard image.
    ///
    /// 1. Reads the image from the clipboard.
    /// 2. Loads the target color from configuration.
    /// 3. Makes matching pixels transparent.
    /// 4. Writes the processed image back to the clipboard.
    ///
    /// Returns `ProcessResult::NoImage` if no image is on the clipboard.
    ///
    /// # Errors
    ///
    /// Returns an error string if any clipboard or config operation fails.
    pub fn process_clipboard(&self) -> Result<ProcessResult, String> {
        let Some(mut image) = self
            .clipboard
            .get_image()
            .map_err(|e| format!("failed to read clipboard: {e}"))?
        else {
            return Ok(ProcessResult::NoImage);
        };

        debug!(
            "image detected on clipboard: {}x{} ({} bytes)",
            image.width,
            image.height,
            image.pixels.len()
        );

        let target_color = self
            .config
            .load_target_color()
            .map_err(|e| format!("failed to load config: {e}"))?;

        debug!(
            "target color loaded: RGB({}, {}, {})",
            target_color.r(),
            target_color.g(),
            target_color.b()
        );

        make_transparent(&mut image.pixels, &target_color);

        self.clipboard
            .set_image(&image)
            .map_err(|e| format!("failed to write clipboard: {e}"))?;

        debug!("transparency applied, image written back to clipboard");

        Ok(ProcessResult::Processed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::color::Color;
    use crate::domain::port::ImageData;
    use std::cell::RefCell;

    // -- Mock ClipboardPort --

    #[derive(Debug)]
    struct MockClipboard {
        image: RefCell<Option<ImageData>>,
    }

    #[derive(Debug)]
    struct MockClipboardError(String);

    impl std::fmt::Display for MockClipboardError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self.0)
        }
    }

    impl std::error::Error for MockClipboardError {}

    impl ClipboardPort for MockClipboard {
        type Error = MockClipboardError;

        fn get_image(&self) -> Result<Option<ImageData>, Self::Error> {
            Ok(self.image.borrow().clone())
        }

        fn set_image(&self, image: &ImageData) -> Result<(), Self::Error> {
            *self.image.borrow_mut() = Some(image.clone());
            Ok(())
        }
    }

    // -- Mock ConfigPort --

    #[derive(Debug)]
    struct MockConfig {
        color: Color,
    }

    #[derive(Debug)]
    struct MockConfigError(String);

    impl std::fmt::Display for MockConfigError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self.0)
        }
    }

    impl std::error::Error for MockConfigError {}

    impl ConfigPort for MockConfig {
        type Error = MockConfigError;

        fn load_target_color(&self) -> Result<Color, Self::Error> {
            Ok(self.color)
        }

        fn save_target_color(&self, _color: &Color) -> Result<(), Self::Error> {
            Ok(())
        }

        fn ensure_config_exists(&self) -> Result<(), Self::Error> {
            Ok(())
        }
    }

    fn make_service(
        image: Option<ImageData>,
        target: Color,
    ) -> ClipboardService<MockClipboard, MockConfig> {
        ClipboardService::new(
            MockClipboard {
                image: RefCell::new(image),
            },
            MockConfig { color: target },
        )
    }

    #[test]
    fn returns_no_image_when_clipboard_empty() {
        let service = make_service(None, Color::default());
        let result = service.process_clipboard().unwrap();
        assert_eq!(result, ProcessResult::NoImage);
    }

    #[test]
    fn processes_image_and_makes_target_transparent() {
        // 2x1 image: white pixel, black pixel
        let image = ImageData {
            pixels: vec![255, 255, 255, 255, 0, 0, 0, 255],
            width: 2,
            height: 1,
        };
        let service = make_service(Some(image), Color::new(255, 255, 255));

        let result = service.process_clipboard().unwrap();
        assert_eq!(result, ProcessResult::Processed);

        // Verify the written image
        let written = service.clipboard.image.borrow();
        let written = written.as_ref().unwrap();
        assert_eq!(written.pixels, vec![255, 255, 255, 0, 0, 0, 0, 255]);
        assert_eq!(written.width, 2);
        assert_eq!(written.height, 1);
    }

    #[test]
    fn non_matching_pixels_unchanged() {
        let image = ImageData {
            pixels: vec![0, 0, 0, 255, 128, 128, 128, 255],
            width: 2,
            height: 1,
        };
        let service = make_service(Some(image), Color::new(255, 255, 255));

        service.process_clipboard().unwrap();

        let written = service.clipboard.image.borrow();
        let written = written.as_ref().unwrap();
        assert_eq!(written.pixels, vec![0, 0, 0, 255, 128, 128, 128, 255]);
    }

    #[test]
    fn uses_configured_target_color() {
        // Target is black, not white
        let image = ImageData {
            pixels: vec![0, 0, 0, 255, 255, 255, 255, 255],
            width: 2,
            height: 1,
        };
        let service = make_service(Some(image), Color::new(0, 0, 0));

        service.process_clipboard().unwrap();

        let written = service.clipboard.image.borrow();
        let written = written.as_ref().unwrap();
        assert_eq!(written.pixels, vec![0, 0, 0, 0, 255, 255, 255, 255]);
    }
}
