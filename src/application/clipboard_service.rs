use std::cell::Cell;

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
    /// The clipboard has not changed since the last check (skipped).
    Skipped,
}

/// Orchestrates the clipboard-to-transparent-image workflow.
///
/// Reads an image from the clipboard, applies transparency conversion
/// for the configured target color, and writes the result back.
///
/// Uses the OS clipboard change counter for lightweight change detection.
/// After writing a processed image back, it records the new counter value
/// so its own write is not re-processed on the next poll.
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
    /// The clipboard change counter after the last write (or initial check).
    last_change_count: Cell<u64>,
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
            last_change_count: Cell::new(0),
        }
    }

    /// Processes the current clipboard image.
    ///
    /// 1. Checks the clipboard change counter (lightweight).
    /// 2. If unchanged, returns `Skipped` without reading the image.
    /// 3. Reads the image from the clipboard.
    /// 4. Loads the target color from configuration.
    /// 5. Makes matching pixels transparent.
    /// 6. Writes the processed image back to the clipboard.
    /// 7. Records the new change counter to avoid re-processing.
    ///
    /// Returns `ProcessResult::NoImage` if no image is on the clipboard.
    ///
    /// # Errors
    ///
    /// Returns an error string if any clipboard or config operation fails.
    pub fn process_clipboard(&self) -> Result<ProcessResult, String> {
        // Step 1: Lightweight change detection via counter
        let current_count = self
            .clipboard
            .change_count()
            .map_err(|e| format!("failed to read change count: {e}"))?;

        if current_count == self.last_change_count.get() {
            return Ok(ProcessResult::Skipped);
        }

        debug!(
            "clipboard changed (count: {} -> {})",
            self.last_change_count.get(),
            current_count
        );

        // Step 2: Read the image
        let Some(mut image) = self
            .clipboard
            .get_image()
            .map_err(|e| format!("failed to read clipboard: {e}"))?
        else {
            // No image — remember this counter so we don't re-check
            self.last_change_count.set(current_count);
            return Ok(ProcessResult::NoImage);
        };

        debug!(
            "image detected on clipboard: {}x{} ({} bytes)",
            image.width,
            image.height,
            image.pixels.len()
        );

        // Sample corner pixel for diagnostics
        if image.pixels.len() >= 4 {
            let (r, g, b, a) = (
                image.pixels[0],
                image.pixels[1],
                image.pixels[2],
                image.pixels[3],
            );
            debug!("sample pixel (0,0): RGBA({r},{g},{b},{a})");
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
            self.last_change_count.set(current_count);
            return Ok(ProcessResult::Processed);
        }

        self.clipboard
            .set_image(&image)
            .map_err(|e| format!("failed to write clipboard: {e}"))?;

        // Record the counter AFTER our write so we skip our own change
        let new_count = self
            .clipboard
            .change_count()
            .map_err(|e| format!("failed to read change count after write: {e}"))?;
        self.last_change_count.set(new_count);

        debug!("transparency applied, image written back to clipboard (count: {new_count})");

        Ok(ProcessResult::Processed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::color::Color;
    use crate::domain::port::ImageData;
    use std::cell::{Cell as StdCell, RefCell};

    // -- Mock ClipboardPort --

    #[derive(Debug)]
    struct MockClipboard {
        image: RefCell<Option<ImageData>>,
        counter: StdCell<u64>,
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

        fn change_count(&self) -> Result<u64, Self::Error> {
            Ok(self.counter.get())
        }

        fn get_image(&self) -> Result<Option<ImageData>, Self::Error> {
            Ok(self.image.borrow().clone())
        }

        fn set_image(&self, image: &ImageData) -> Result<(), Self::Error> {
            *self.image.borrow_mut() = Some(image.clone());
            // Increment counter to simulate OS behavior
            self.counter.set(self.counter.get() + 1);
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
                // Start at 1 so it differs from the initial last_change_count of 0
                counter: StdCell::new(1),
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
        let image = ImageData {
            pixels: vec![255, 255, 255, 255, 0, 0, 0, 255],
            width: 2,
            height: 1,
        };
        let service = make_service(Some(image), Color::new(255, 255, 255));

        let result = service.process_clipboard().unwrap();
        assert_eq!(result, ProcessResult::Processed);

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

        // No pixels matched, so image is unchanged in clipboard
        let written = service.clipboard.image.borrow();
        let written = written.as_ref().unwrap();
        assert_eq!(written.pixels, vec![0, 0, 0, 255, 128, 128, 128, 255]);
    }

    #[test]
    fn uses_configured_target_color() {
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
    fn skips_when_change_count_unchanged() {
        let image = ImageData {
            pixels: vec![255, 255, 255, 255, 0, 0, 0, 255],
            width: 2,
            height: 1,
        };
        let service = make_service(Some(image), Color::new(255, 255, 255));

        // First call processes (counter=1 != last=0)
        let result = service.process_clipboard().unwrap();
        assert_eq!(result, ProcessResult::Processed);

        // Second call: counter was updated after set_image, so it matches last
        let result = service.process_clipboard().unwrap();
        assert_eq!(result, ProcessResult::Skipped);
    }

    #[test]
    fn processes_after_external_clipboard_change() {
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

        // Simulate external clipboard change: new image + bump counter
        *service.clipboard.image.borrow_mut() = Some(ImageData {
            pixels: vec![100, 100, 100, 255],
            width: 1,
            height: 1,
        });
        service
            .clipboard
            .counter
            .set(service.clipboard.counter.get() + 1);

        // Should process the new image
        assert_eq!(
            service.process_clipboard().unwrap(),
            ProcessResult::Processed
        );
    }
}
