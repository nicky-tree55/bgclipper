use bgclipper::domain::color::Color;
use bgclipper::domain::image_processor::make_transparent;

/// Golden test: apply transparency to logo input and compare with expected output.
///
/// - `tests/fixtures/input.png`: logo with solid background RGB(231, 254, 182)
/// - `tests/fixtures/expected.png`: same logo with that background made transparent
#[test]
fn logo_background_becomes_transparent() {
    let input_bytes = include_bytes!("fixtures/input.png");
    let expected_bytes = include_bytes!("fixtures/expected.png");

    // Decode input PNG to RGBA pixels
    let input_image = image::load_from_memory(input_bytes).expect("failed to load input.png");
    let mut input_rgba = input_image.to_rgba8();
    let (width, height) = input_rgba.dimensions();

    // Decode expected PNG to RGBA pixels
    let expected_image =
        image::load_from_memory(expected_bytes).expect("failed to load expected.png");
    let expected_rgba = expected_image.to_rgba8();
    let (ew, eh) = expected_rgba.dimensions();

    assert_eq!(
        (width, height),
        (ew, eh),
        "input and expected images must have the same dimensions"
    );

    // Apply transparency for background color #e7feb6
    let bg_color = Color::new(231, 254, 182);
    make_transparent(input_rgba.as_mut(), &bg_color);

    // Compare pixel by pixel (for fully transparent pixels, ignore RGB values).
    // Allow a small number of mismatches at anti-aliased edges where the
    // renderer blends the background color with shape edges.
    let actual = input_rgba.as_raw();
    let expected = expected_rgba.as_raw();

    assert_eq!(
        actual.len(),
        expected.len(),
        "pixel buffer sizes must match"
    );

    let mut mismatches = 0;
    let mut first_mismatch = None;
    for i in (0..actual.len()).step_by(4) {
        let (ar, ag, ab, aa) = (actual[i], actual[i + 1], actual[i + 2], actual[i + 3]);
        let (er, eg, eb, ea) = (
            expected[i],
            expected[i + 1],
            expected[i + 2],
            expected[i + 3],
        );

        // If both are fully transparent, RGB doesn't matter
        if aa == 0 && ea == 0 {
            continue;
        }

        let diff = ar != er || ag != eg || ab != eb || aa != ea;
        if diff {
            mismatches += 1;
            if first_mismatch.is_none() {
                let pixel_idx = i / 4;
                let x = pixel_idx % width as usize;
                let y = pixel_idx / width as usize;
                first_mismatch = Some(format!(
                    "pixel ({x},{y}): actual=RGBA({ar},{ag},{ab},{aa}), expected=RGBA({er},{eg},{eb},{ea})"
                ));
            }
        }
    }

    // Anti-aliased edges produce a small number of near-background pixels
    // that don't exactly match #e7feb6. Allow up to 1% of total pixels.
    let total_pixels = (width * height) as usize;
    let tolerance = total_pixels / 100;
    assert!(
        mismatches <= tolerance,
        "{mismatches} pixel(s) differ (tolerance: {tolerance}). First mismatch: {}",
        first_mismatch.unwrap_or_default()
    );
}
