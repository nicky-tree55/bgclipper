use crate::domain::color::Color;

/// Replaces pixels matching the target color with full transparency.
///
/// Scans the RGBA pixel buffer and sets the alpha channel to `0` for every
/// pixel whose RGB channels exactly match `target`. Non-matching pixels
/// are left unchanged.
///
/// Returns the number of pixels that were made transparent.
///
/// # Arguments
///
/// * `pixels` - Mutable RGBA pixel buffer (4 bytes per pixel: R, G, B, A).
/// * `target` - The color to make transparent.
///
/// # Panics
///
/// Panics if `pixels.len()` is not a multiple of 4.
///
/// # Examples
///
/// ```
/// use bgclipper::domain::color::Color;
/// use bgclipper::domain::image_processor::make_transparent;
///
/// let mut pixels = vec![255, 255, 255, 255, 0, 0, 0, 255];
/// let white = Color::new(255, 255, 255);
/// let count = make_transparent(&mut pixels, &white);
/// assert_eq!(count, 1);
/// assert_eq!(pixels, vec![255, 255, 255, 0, 0, 0, 0, 255]);
/// ```
pub fn make_transparent(pixels: &mut [u8], target: &Color) -> usize {
    assert!(
        pixels.len().is_multiple_of(4),
        "pixel buffer length must be a multiple of 4, got {}",
        pixels.len()
    );

    let mut count = 0;
    for chunk in pixels.chunks_exact_mut(4) {
        let pixel_color = Color::new(chunk[0], chunk[1], chunk[2]);
        if pixel_color.matches(target) {
            chunk[3] = 0;
            count += 1;
        }
    }
    count
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matching_pixels_become_transparent() {
        let mut pixels = vec![255, 255, 255, 255];
        let target = Color::new(255, 255, 255);
        make_transparent(&mut pixels, &target);
        assert_eq!(pixels, vec![255, 255, 255, 0]);
    }

    #[test]
    fn non_matching_pixels_are_unchanged() {
        let mut pixels = vec![0, 0, 0, 255];
        let target = Color::new(255, 255, 255);
        make_transparent(&mut pixels, &target);
        assert_eq!(pixels, vec![0, 0, 0, 255]);
    }

    #[test]
    fn mixed_pixels() {
        // white, black, white, red
        let mut pixels = vec![
            255, 255, 255, 255, // white -> transparent
            0, 0, 0, 255, // black -> unchanged
            255, 255, 255, 255, // white -> transparent
            255, 0, 0, 255, // red -> unchanged
        ];
        let target = Color::new(255, 255, 255);
        make_transparent(&mut pixels, &target);
        assert_eq!(
            pixels,
            vec![
                255, 255, 255, 0, 0, 0, 0, 255, 255, 255, 255, 0, 255, 0, 0, 255,
            ]
        );
    }

    #[test]
    fn already_transparent_pixel_stays_transparent() {
        let mut pixels = vec![255, 255, 255, 0];
        let target = Color::new(255, 255, 255);
        make_transparent(&mut pixels, &target);
        assert_eq!(pixels, vec![255, 255, 255, 0]);
    }

    #[test]
    fn empty_buffer() {
        let mut pixels: Vec<u8> = vec![];
        let target = Color::new(255, 255, 255);
        make_transparent(&mut pixels, &target);
        assert!(pixels.is_empty());
    }

    #[test]
    #[should_panic(expected = "pixel buffer length must be a multiple of 4")]
    fn invalid_buffer_length_panics() {
        let mut pixels = vec![255, 255, 255];
        let target = Color::new(255, 255, 255);
        make_transparent(&mut pixels, &target);
    }

    #[test]
    fn target_black() {
        let mut pixels = vec![
            0, 0, 0, 255, // black -> transparent
            255, 255, 255, 255, // white -> unchanged
        ];
        let target = Color::new(0, 0, 0);
        make_transparent(&mut pixels, &target);
        assert_eq!(pixels, vec![0, 0, 0, 0, 255, 255, 255, 255,]);
    }

    #[test]
    fn partial_rgb_match_is_not_transparent() {
        // Only 2 of 3 channels match — should NOT be made transparent
        let mut pixels = vec![255, 255, 0, 255];
        let target = Color::new(255, 255, 255);
        make_transparent(&mut pixels, &target);
        assert_eq!(pixels, vec![255, 255, 0, 255]);
    }
}
