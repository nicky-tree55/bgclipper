use std::fmt;

/// Represents an RGB color value.
///
/// Each channel (`r`, `g`, `b`) is stored as a `u8` (0–255).
/// Used to specify the target background color for transparency conversion.
///
/// # Examples
///
/// ```
/// use bgclipper::domain::color::Color;
///
/// let white = Color::new(255, 255, 255);
/// assert_eq!(white.r(), 255);
/// assert_eq!(white.g(), 255);
/// assert_eq!(white.b(), 255);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Color {
    r: u8,
    g: u8,
    b: u8,
}

impl Color {
    /// Creates a new `Color` with the given RGB channel values.
    pub fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    /// Returns the red channel value.
    pub fn r(&self) -> u8 {
        self.r
    }

    /// Returns the green channel value.
    pub fn g(&self) -> u8 {
        self.g
    }

    /// Returns the blue channel value.
    pub fn b(&self) -> u8 {
        self.b
    }

    /// Returns `true` if this color matches the given color exactly.
    pub fn matches(&self, other: &Color) -> bool {
        self == other
    }
}

impl Default for Color {
    /// Returns white (`rgb(255, 255, 255)`) as the default color.
    fn default() -> Self {
        Self::new(255, 255, 255)
    }
}

impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "rgb({}, {}, {})", self.r, self.g, self.b)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_creates_color_with_given_values() {
        let color = Color::new(10, 20, 30);
        assert_eq!(color.r(), 10);
        assert_eq!(color.g(), 20);
        assert_eq!(color.b(), 30);
    }

    #[test]
    fn default_returns_white() {
        let color = Color::default();
        assert_eq!(color.r(), 255);
        assert_eq!(color.g(), 255);
        assert_eq!(color.b(), 255);
    }

    #[test]
    fn equal_colors_are_equal() {
        let a = Color::new(100, 150, 200);
        let b = Color::new(100, 150, 200);
        assert_eq!(a, b);
    }

    #[test]
    fn different_colors_are_not_equal() {
        let a = Color::new(100, 150, 200);
        let b = Color::new(100, 150, 201);
        assert_ne!(a, b);
    }

    #[test]
    fn display_formats_as_rgb() {
        let color = Color::new(255, 128, 0);
        assert_eq!(format!("{color}"), "rgb(255, 128, 0)");
    }

    #[test]
    fn matches_returns_true_for_same_color() {
        let a = Color::new(0, 0, 0);
        let b = Color::new(0, 0, 0);
        assert!(a.matches(&b));
    }

    #[test]
    fn matches_returns_false_for_different_color() {
        let a = Color::new(0, 0, 0);
        let b = Color::new(255, 255, 255);
        assert!(!a.matches(&b));
    }

    #[test]
    fn copy_semantics_work() {
        let a = Color::new(1, 2, 3);
        let b = a;
        // `a` is still usable after copy
        assert_eq!(a, b);
    }

    #[test]
    fn clone_produces_equal_value() {
        let a = Color::new(50, 100, 150);
        #[allow(clippy::clone_on_copy)]
        let b = a.clone();
        assert_eq!(a, b);
    }

    #[test]
    fn boundary_values() {
        let min = Color::new(0, 0, 0);
        assert_eq!(min.r(), 0);
        assert_eq!(min.g(), 0);
        assert_eq!(min.b(), 0);

        let max = Color::new(255, 255, 255);
        assert_eq!(max.r(), 255);
        assert_eq!(max.g(), 255);
        assert_eq!(max.b(), 255);
    }

    #[test]
    fn hash_is_consistent_for_equal_colors() {
        use std::collections::HashSet;

        let a = Color::new(10, 20, 30);
        let b = Color::new(10, 20, 30);

        let mut set = HashSet::new();
        set.insert(a);
        assert!(set.contains(&b));
    }
}
