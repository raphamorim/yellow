/// Terminal colors
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Color {
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    White,
    BrightBlack,
    BrightRed,
    BrightGreen,
    BrightYellow,
    BrightBlue,
    BrightMagenta,
    BrightCyan,
    BrightWhite,
    Rgb(u8, u8, u8),
    Ansi256(u8),
    /// Reset to terminal default color
    Reset,
}

impl Color {
    /// Convert color to (discriminant, data) for efficient hashing
    /// Returns (type_byte, data_u32) to minimize branches in hash functions
    #[inline(always)]
    pub(crate) fn hash_bytes(&self) -> (u8, u32) {
        match self {
            Color::Black => (1, 0),
            Color::Red => (2, 0),
            Color::Green => (3, 0),
            Color::Yellow => (4, 0),
            Color::Blue => (5, 0),
            Color::Magenta => (6, 0),
            Color::Cyan => (7, 0),
            Color::White => (8, 0),
            Color::BrightBlack => (9, 0),
            Color::BrightRed => (10, 0),
            Color::BrightGreen => (11, 0),
            Color::BrightYellow => (12, 0),
            Color::BrightBlue => (13, 0),
            Color::BrightMagenta => (14, 0),
            Color::BrightCyan => (15, 0),
            Color::BrightWhite => (16, 0),
            Color::Ansi256(c) => (17, *c as u32),
            Color::Rgb(r, g, b) => (18, ((*r as u32) << 16) | ((*g as u32) << 8) | (*b as u32)),
            Color::Reset => (19, 0),
        }
    }

    /// Write foreground ANSI code directly to a string buffer (zero-allocation for basic colors)
    pub(crate) fn write_ansi_fg(&self, buf: &mut String) {
        use std::fmt::Write;
        match self {
            Color::Black => buf.push_str("30"),
            Color::Red => buf.push_str("31"),
            Color::Green => buf.push_str("32"),
            Color::Yellow => buf.push_str("33"),
            Color::Blue => buf.push_str("34"),
            Color::Magenta => buf.push_str("35"),
            Color::Cyan => buf.push_str("36"),
            Color::White => buf.push_str("37"),
            Color::BrightBlack => buf.push_str("90"),
            Color::BrightRed => buf.push_str("91"),
            Color::BrightGreen => buf.push_str("92"),
            Color::BrightYellow => buf.push_str("93"),
            Color::BrightBlue => buf.push_str("94"),
            Color::BrightMagenta => buf.push_str("95"),
            Color::BrightCyan => buf.push_str("96"),
            Color::BrightWhite => buf.push_str("97"),
            Color::Rgb(r, g, b) => write!(buf, "38;2;{};{};{}", r, g, b).unwrap(),
            Color::Ansi256(c) => write!(buf, "38;5;{}", c).unwrap(),
            Color::Reset => buf.push_str("39"),
        }
    }

    /// Write background ANSI code directly to a string buffer (zero-allocation for basic colors)
    pub(crate) fn write_ansi_bg(&self, buf: &mut String) {
        use std::fmt::Write;
        match self {
            Color::Black => buf.push_str("40"),
            Color::Red => buf.push_str("41"),
            Color::Green => buf.push_str("42"),
            Color::Yellow => buf.push_str("43"),
            Color::Blue => buf.push_str("44"),
            Color::Magenta => buf.push_str("45"),
            Color::Cyan => buf.push_str("46"),
            Color::White => buf.push_str("47"),
            Color::BrightBlack => buf.push_str("100"),
            Color::BrightRed => buf.push_str("101"),
            Color::BrightGreen => buf.push_str("102"),
            Color::BrightYellow => buf.push_str("103"),
            Color::BrightBlue => buf.push_str("104"),
            Color::BrightMagenta => buf.push_str("105"),
            Color::BrightCyan => buf.push_str("106"),
            Color::BrightWhite => buf.push_str("107"),
            Color::Rgb(r, g, b) => write!(buf, "48;2;{};{};{}", r, g, b).unwrap(),
            Color::Ansi256(c) => write!(buf, "48;5;{}", c).unwrap(),
            Color::Reset => buf.push_str("49"),
        }
    }

    // Keep old methods for backward compatibility (used in tests and mosaic)
    pub(crate) fn to_ansi_fg(&self) -> String {
        let mut buf = String::with_capacity(16);
        self.write_ansi_fg(&mut buf);
        buf
    }

    pub(crate) fn to_ansi_bg(&self) -> String {
        let mut buf = String::with_capacity(16);
        self.write_ansi_bg(&mut buf);
        buf
    }
}

/// A color pair consisting of foreground and background colors
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ColorPair {
    pub fg: Color,
    pub bg: Color,
}

impl ColorPair {
    pub fn new(fg: Color, bg: Color) -> Self {
        Self { fg, bg }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_ansi_fg() {
        assert_eq!(Color::Red.to_ansi_fg(), "31");
        assert_eq!(Color::BrightBlue.to_ansi_fg(), "94");
        assert_eq!(Color::Rgb(255, 128, 0).to_ansi_fg(), "38;2;255;128;0");
        assert_eq!(Color::Ansi256(42).to_ansi_fg(), "38;5;42");
    }

    #[test]
    fn test_color_ansi_bg() {
        assert_eq!(Color::Green.to_ansi_bg(), "42");
        assert_eq!(Color::BrightMagenta.to_ansi_bg(), "105");
        assert_eq!(Color::Rgb(0, 128, 255).to_ansi_bg(), "48;2;0;128;255");
        assert_eq!(Color::Ansi256(100).to_ansi_bg(), "48;5;100");
    }

    #[test]
    fn test_color_pair() {
        let pair = ColorPair::new(Color::Red, Color::Black);
        assert_eq!(pair.fg, Color::Red);
        assert_eq!(pair.bg, Color::Black);
    }

    #[test]
    fn test_color_equality() {
        assert_eq!(Color::Red, Color::Red);
        assert_ne!(Color::Red, Color::Blue);
        assert_eq!(Color::Rgb(255, 0, 0), Color::Rgb(255, 0, 0));
        assert_ne!(Color::Rgb(255, 0, 0), Color::Rgb(255, 0, 1));
    }

    #[test]
    fn test_color_reset() {
        assert_eq!(Color::Reset.to_ansi_fg(), "39");
        assert_eq!(Color::Reset.to_ansi_bg(), "49");
    }
}
