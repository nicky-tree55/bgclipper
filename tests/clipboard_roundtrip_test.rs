use std::borrow::Cow;

use arboard::Clipboard;
use bgclipper::application::clipboard_service::{ClipboardService, ProcessResult};
use bgclipper::domain::color::Color;
use bgclipper::domain::port::ConfigPort;
use bgclipper::infrastructure::clipboard::ArboardClipboardProvider;

// -- Inline ConfigPort for testing (returns a fixed color) --

#[derive(Debug)]
struct FixedConfig {
    color: Color,
}

#[derive(Debug)]
struct FixedConfigError;

impl std::fmt::Display for FixedConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "fixed config error")
    }
}

impl std::error::Error for FixedConfigError {}

impl ConfigPort for FixedConfig {
    type Error = FixedConfigError;

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

/// End-to-end test: set a known image on the real clipboard, run the service,
/// then read the clipboard back and verify that the target-color pixels have
/// alpha = 0.
///
/// This test uses the real system clipboard, so it must NOT run in parallel
/// with other clipboard tests.
#[test]
fn clipboard_roundtrip_preserves_transparency() {
    // 2x2 image: top-left = background (red), rest = other colors
    // Target: red RGB(255, 0, 0)
    #[rustfmt::skip]
    let input_pixels: Vec<u8> = vec![
        255,   0,   0, 255, //  (0,0) red   -> should become transparent
          0, 255,   0, 255, //  (1,0) green -> unchanged
          0,   0, 255, 255, //  (0,1) blue  -> unchanged
        255,   0,   0, 255, //  (1,1) red   -> should become transparent
    ];
    let width: u32 = 2;
    let height: u32 = 2;

    // Step 1: Write the input image to the real clipboard
    {
        let mut clipboard = Clipboard::new().expect("failed to open clipboard");
        let img = arboard::ImageData {
            width: width as usize,
            height: height as usize,
            bytes: Cow::Borrowed(&input_pixels),
        };
        clipboard.set_image(img).expect("failed to set image");
    }

    // Step 2: Run the service with real clipboard + fixed config
    let provider = ArboardClipboardProvider::new();
    let config = FixedConfig {
        color: Color::new(255, 0, 0),
    };
    let service = ClipboardService::new(provider, config);

    let result = service
        .process_clipboard()
        .expect("process_clipboard failed");
    assert_eq!(result, ProcessResult::Processed);

    // Step 3: Read the image back from the real clipboard
    let mut clipboard = Clipboard::new().expect("failed to open clipboard");
    let output = clipboard.get_image().expect("failed to get image");

    assert_eq!(output.width, width as usize);
    assert_eq!(output.height, height as usize);

    let px = output.bytes;
    assert_eq!(px.len(), 16, "expected 2x2 RGBA = 16 bytes");

    // Pixel (0,0): was red -> alpha must be 0
    assert_eq!(px[3], 0, "pixel (0,0) alpha should be 0, got {}", px[3]);

    // Pixel (1,0): was green -> unchanged
    assert_eq!(px[4], 0, "pixel (1,0) R");
    assert_eq!(px[5], 255, "pixel (1,0) G");
    assert_eq!(px[6], 0, "pixel (1,0) B");
    assert_eq!(px[7], 255, "pixel (1,0) A should remain 255");

    // Pixel (0,1): was blue -> unchanged
    assert_eq!(px[8], 0, "pixel (0,1) R");
    assert_eq!(px[9], 0, "pixel (0,1) G");
    assert_eq!(px[10], 255, "pixel (0,1) B");
    assert_eq!(px[11], 255, "pixel (0,1) A should remain 255");

    // Pixel (1,1): was red -> alpha must be 0
    assert_eq!(px[15], 0, "pixel (1,1) alpha should be 0, got {}", px[15]);
}
