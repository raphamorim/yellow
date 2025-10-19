/// Packed color representation for memory efficiency
///
/// Encodes an Option<Color> in 16 bits:
/// - Bit 15: has_color (1 = color present)
/// - Bits 14-12: color_type
/// - Bits 11-0: color_data
use crate::color::Color;

#[repr(transparent)]
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct PackedColor(u16);

impl PackedColor {
    // Bit masks
    const HAS_COLOR: u16 = 1 << 15;
    const TYPE_SHIFT: u16 = 12;
    const TYPE_MASK: u16 = 0b111 << Self::TYPE_SHIFT;
    const DATA_MASK: u16 = 0xFFF;

    // Color types
    const TYPE_BASIC: u16 = 1;
    const TYPE_256: u16 = 2;
    const TYPE_RGB: u16 = 3;

    /// Create a PackedColor representing no color
    pub const fn none() -> Self {
        Self(0)
    }

    /// Check if this represents a color (vs None)
    pub const fn has_color(self) -> bool {
        (self.0 & Self::HAS_COLOR) != 0
    }

    /// Pack an Option<Color> into 16 bits
    pub fn from_color(color: Option<Color>) -> Self {
        match color {
            None => Self::none(),
            Some(Color::Black) => Self::from_basic(0),
            Some(Color::Red) => Self::from_basic(1),
            Some(Color::Green) => Self::from_basic(2),
            Some(Color::Yellow) => Self::from_basic(3),
            Some(Color::Blue) => Self::from_basic(4),
            Some(Color::Magenta) => Self::from_basic(5),
            Some(Color::Cyan) => Self::from_basic(6),
            Some(Color::White) => Self::from_basic(7),
            Some(Color::BrightBlack) => Self::from_basic(8),
            Some(Color::BrightRed) => Self::from_basic(9),
            Some(Color::BrightGreen) => Self::from_basic(10),
            Some(Color::BrightYellow) => Self::from_basic(11),
            Some(Color::BrightBlue) => Self::from_basic(12),
            Some(Color::BrightMagenta) => Self::from_basic(13),
            Some(Color::BrightCyan) => Self::from_basic(14),
            Some(Color::BrightWhite) => Self::from_basic(15),
            Some(Color::Ansi256(c)) => Self::from_256(c),
            Some(Color::Rgb(r, g, b)) => Self::from_rgb(r, g, b),
            Some(Color::Reset) => Self::none(), // Reset is treated as no color (terminal default)
        }
    }

    /// Unpack to Option<Color>
    pub fn to_color(self) -> Option<Color> {
        if !self.has_color() {
            return None;
        }

        let color_type = (self.0 & Self::TYPE_MASK) >> Self::TYPE_SHIFT;
        let data = self.0 & Self::DATA_MASK;

        match color_type {
            Self::TYPE_BASIC => Some(Self::decode_basic(data as u8)),
            Self::TYPE_256 => Some(Color::Ansi256(data as u8)),
            Self::TYPE_RGB => Some(Self::decode_rgb(data)),
            _ => None, // Invalid type, treat as no color
        }
    }

    // Helper constructors
    fn from_basic(index: u8) -> Self {
        debug_assert!(index < 16, "Basic color index must be 0-15");
        Self(Self::HAS_COLOR | (Self::TYPE_BASIC << Self::TYPE_SHIFT) | (index as u16))
    }

    fn from_256(index: u8) -> Self {
        Self(Self::HAS_COLOR | (Self::TYPE_256 << Self::TYPE_SHIFT) | (index as u16))
    }

    fn from_rgb(r: u8, g: u8, b: u8) -> Self {
        // Compress to 4 bits per channel (16 levels)
        let r4 = (r >> 4) as u16;
        let g4 = (g >> 4) as u16;
        let b4 = (b >> 4) as u16;
        let data = (r4 << 8) | (g4 << 4) | b4;
        Self(Self::HAS_COLOR | (Self::TYPE_RGB << Self::TYPE_SHIFT) | data)
    }

    // Helper decoders
    fn decode_basic(index: u8) -> Color {
        match index {
            0 => Color::Black,
            1 => Color::Red,
            2 => Color::Green,
            3 => Color::Yellow,
            4 => Color::Blue,
            5 => Color::Magenta,
            6 => Color::Cyan,
            7 => Color::White,
            8 => Color::BrightBlack,
            9 => Color::BrightRed,
            10 => Color::BrightGreen,
            11 => Color::BrightYellow,
            12 => Color::BrightBlue,
            13 => Color::BrightMagenta,
            14 => Color::BrightCyan,
            15 => Color::BrightWhite,
            _ => Color::White, // Fallback
        }
    }

    fn decode_rgb(data: u16) -> Color {
        // Expand 4 bits per channel back to 8 bits
        let r4 = ((data >> 8) & 0xF) as u8;
        let g4 = ((data >> 4) & 0xF) as u8;
        let b4 = (data & 0xF) as u8;

        // Expand: duplicate high nibble to low nibble for smoothness
        let r = (r4 << 4) | r4;
        let g = (g4 << 4) | g4;
        let b = (b4 << 4) | b4;

        Color::Rgb(r, g, b)
    }
}

impl Default for PackedColor {
    fn default() -> Self {
        Self::none()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_packed_color_size() {
        // PackedColor is repr(transparent) wrapping u16, so it should be exactly 2 bytes
        let size = std::mem::size_of::<PackedColor>();
        assert_eq!(size, 2, "PackedColor should be exactly 2 bytes (u16)");
        assert_eq!(size, std::mem::size_of::<u16>(), "PackedColor size should match u16");
    }

    #[test]
    fn test_none() {
        let pc = PackedColor::none();
        assert!(!pc.has_color());
        assert_eq!(pc.to_color(), None);
    }

    #[test]
    fn test_basic_colors() {
        let colors = vec![
            Color::Black,
            Color::Red,
            Color::Green,
            Color::Yellow,
            Color::Blue,
            Color::Magenta,
            Color::Cyan,
            Color::White,
        ];

        for color in colors {
            let packed = PackedColor::from_color(Some(color));
            assert!(packed.has_color());
            assert_eq!(packed.to_color(), Some(color));
        }
    }

    #[test]
    fn test_bright_colors() {
        let colors = vec![
            Color::BrightBlack,
            Color::BrightRed,
            Color::BrightGreen,
            Color::BrightYellow,
            Color::BrightBlue,
            Color::BrightMagenta,
            Color::BrightCyan,
            Color::BrightWhite,
        ];

        for color in colors {
            let packed = PackedColor::from_color(Some(color));
            assert!(packed.has_color());
            assert_eq!(packed.to_color(), Some(color));
        }
    }

    #[test]
    fn test_ansi256() {
        for i in 0..=255 {
            let color = Color::Ansi256(i);
            let packed = PackedColor::from_color(Some(color));
            assert!(packed.has_color());
            assert_eq!(packed.to_color(), Some(color));
        }
    }

    #[test]
    fn test_rgb_roundtrip() {
        // Test exact values at 4-bit boundaries
        let test_cases = vec![
            (0, 0, 0),
            (255, 255, 255),
            (255, 0, 0),
            (0, 255, 0),
            (0, 0, 255),
            (128, 128, 128),
        ];

        for (r, g, b) in test_cases {
            let color = Color::Rgb(r, g, b);
            let packed = PackedColor::from_color(Some(color));
            assert!(packed.has_color());

            match packed.to_color() {
                Some(Color::Rgb(r2, g2, b2)) => {
                    // Allow for 4-bit quantization error (max 15 per channel)
                    assert!((r as i16 - r2 as i16).abs() <= 17,
                           "R mismatch: {} vs {}", r, r2);
                    assert!((g as i16 - g2 as i16).abs() <= 17,
                           "G mismatch: {} vs {}", g, g2);
                    assert!((b as i16 - b2 as i16).abs() <= 17,
                           "B mismatch: {} vs {}", b, b2);
                }
                other => panic!("Expected RGB, got {:?}", other),
            }
        }
    }

    #[test]
    fn test_rgb_precision() {
        // RGB colors are compressed to 4 bits per channel
        // Verify the quantization is reasonable
        let color = Color::Rgb(127, 127, 127); // Mid-gray
        let packed = PackedColor::from_color(Some(color));

        match packed.to_color() {
            Some(Color::Rgb(r, g, b)) => {
                // Should be close to original (within quantization error)
                assert!((127i16 - r as i16).abs() <= 17);
                assert!((127i16 - g as i16).abs() <= 17);
                assert!((127i16 - b as i16).abs() <= 17);
            }
            other => panic!("Expected RGB, got {:?}", other),
        }
    }

    #[test]
    fn test_none_from_option() {
        let packed = PackedColor::from_color(None);
        assert!(!packed.has_color());
        assert_eq!(packed.to_color(), None);
    }

    #[test]
    fn test_default() {
        let packed = PackedColor::default();
        assert!(!packed.has_color());
        assert_eq!(packed.to_color(), None);
    }

    #[test]
    fn test_equality() {
        let c1 = PackedColor::from_color(Some(Color::Red));
        let c2 = PackedColor::from_color(Some(Color::Red));
        let c3 = PackedColor::from_color(Some(Color::Blue));
        let c4 = PackedColor::from_color(None);

        assert_eq!(c1, c2);
        assert_ne!(c1, c3);
        assert_ne!(c1, c4);
    }

    #[test]
    fn test_clone_copy() {
        let c1 = PackedColor::from_color(Some(Color::Green));
        let c2 = c1; // Should copy, not move
        assert_eq!(c1, c2);
    }
}
