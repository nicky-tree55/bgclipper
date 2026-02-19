use std::cell::Cell;
use std::hash::{DefaultHasher, Hash, Hasher};

use log::debug;

use crate::domain::image_processor::make_transparent;
use crate::domain::port::{ClipboardPort, ConfigPort, ImageData};

/// Result of processing a clipboard image.
#[derive(Debug, PartialEq, Eq)]
pub enum ProcessResult {
    /// An image was found and processed successfully.
    Processed,
    /// No image was found on the clipboard.
    NoImage,
    /// The clipboard image was already processed (skipped).
    Skipped,
}

/// Orchestrates the clipboard-to-transparent-image workflow.
///
/// Reads an image from the clipboard, applies transparency conversion
/// for the configured target color, and writes the result back.
///
/// Tracks a hash of the last processed image to avoid re-processing
/// the same image repeatedly (which would cause an infinite loop since
/// the processed image is written back to the clipboard).
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
    /// Hash of the last image written back to the clipboard.
    last_hash: Cell<u64>,
}

impl<C, G> ClipboardService<C, G>
where
    C: ClipboardPort,
    G: ConfigPort,
{
    /// Creates a new service with the given clipboard and config providers.
    pub fn new(clipboard: C, config: G) -> Self {
        Self {
            clipboard,
            config,
            last_hash: Cell::new(0),
        }
    }

    /// Computes a hash of the image data for change detection.
    fn hash_image(image: &ImageData) -> u64 {
        let mut hasher = DefaultHasher::new();
        image.pixels.hash(&mut hasher);
        image.width.hash(&mut hasher);
        image.height.hash(&mut hasher);
        hasher.finish()
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

        // Skip if this image was already processed (avoids infinite loop)
        let incoming_hash = Self::hash_image(&image);
        if incoming_hash == self.last_hash.get() {
            return Ok(ProcessResult::Skipped);
        }

        debug!(
            "image detected on clipboard: {}x{} ({} bytes)",
            image.width,
            image.height,
            image.pixels.len()
        );

        // Sample corner pixels for diagnostics
        if image.pixels.len() >= 4 {
            let (r, g, b, a) = (
                image.pixels[0],
                image.pixels[1],
                image.pixels[2],
                image.pixels[3],
            );
            debug!("sample pixel (0,0): RGBA({r},{g},{b},{a})");
        }
        if image.pixels.len() >= 8 {
            let off = 4;
            let (r, g, b, a) = (
                image.pixels[off],
                image.pixels[off + 1],
                image.pixels[off + 2],
                image.pixels[off + 3],
            );
            debug!("sample pixel (1,0): RGBA({r},{g},{b},{a})");
        }
        // Sample center pixel
        {
            let center = ((image.height / 2) * image.width + (image.width / 2)) as usize * 4;
            if center + 3 < image.pixels.len() {
                let (r, g, b, a) = (
                    image.pixels[center],
                    image.pixels[center + 1],
                    image.pixels[center + 2],
                    image.pixels[center + 3],
                );
                debug!("sample pixel (center): RGBA({r},{g},{b},{a})");
            }
        }

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

        let changed = make_transparent(&mut image.pixels, &target_color);

        debug!("{changed} pixel(s) matched target color");

        if changed == 0 {
            debug!("no pixels matched — skipping clipboard write");
            // Still remember the hash to avoid reprocessing
            self.last_hash.set(Self::hash_image(&image));
            return Ok(ProcessResult::Processed);
        }

        // Remember the hash of the processed image before writing it back
        self.last_hash.set(Self::hash_image(&image));

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

    #[test]
    fn skips_already_processed_image() {
        let image = ImageData {
            pixels: vec![255, 255, 255, 255, 0, 0, 0, 255],
            width: 2,
            height: 1,
        };
        let service = make_service(Some(image), Color::new(255, 255, 255));

        // First call processes
        let result = service.process_clipboard().unwrap();
        assert_eq!(result, ProcessResult::Processed);

        // Second call skips (the processed image is still on the clipboard)
        let result = service.process_clipboard().unwrap();
        assert_eq!(result, ProcessResult::Skipped);
    }

    #[test]
    fn processes_new_image_after_skip() {
        let image1 = ImageData {
            pixels: vec![255, 255, 255, 255],
            width: 1,
            height: 1,
        };
        let service = make_service(Some(image1), Color::new(255, 255, 255));

        assert_eq!(
            service.process_clipboard().unwrap(),
            ProcessResult::Processed
        );
        assert_eq!(service.process_clipboard().unwrap(), ProcessResult::Skipped);

        // Put a new different image on the clipboard
        *service.clipboard.image.borrow_mut() = Some(ImageData {
            pixels: vec![100, 100, 100, 255],
            width: 1,
            height: 1,
        });

        // Should process the new image
        assert_eq!(
            service.process_clipboard().unwrap(),
            ProcessResult::Processed
        );
    }
}
