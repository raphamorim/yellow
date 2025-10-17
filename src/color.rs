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
}

impl Color {
    pub(crate) fn to_ansi_fg(&self) -> String {
        match self {
            Color::Black => "30".to_string(),
            Color::Red => "31".to_string(),
            Color::Green => "32".to_string(),
            Color::Yellow => "33".to_string(),
            Color::Blue => "34".to_string(),
            Color::Magenta => "35".to_string(),
            Color::Cyan => "36".to_string(),
            Color::White => "37".to_string(),
            Color::BrightBlack => "90".to_string(),
            Color::BrightRed => "91".to_string(),
            Color::BrightGreen => "92".to_string(),
            Color::BrightYellow => "93".to_string(),
            Color::BrightBlue => "94".to_string(),
            Color::BrightMagenta => "95".to_string(),
            Color::BrightCyan => "96".to_string(),
            Color::BrightWhite => "97".to_string(),
            Color::Rgb(r, g, b) => format!("38;2;{};{};{}", r, g, b),
            Color::Ansi256(c) => format!("38;5;{}", c),
        }
    }

    pub(crate) fn to_ansi_bg(&self) -> String {
        match self {
            Color::Black => "40".to_string(),
            Color::Red => "41".to_string(),
            Color::Green => "42".to_string(),
            Color::Yellow => "43".to_string(),
            Color::Blue => "44".to_string(),
            Color::Magenta => "45".to_string(),
            Color::Cyan => "46".to_string(),
            Color::White => "47".to_string(),
            Color::BrightBlack => "100".to_string(),
            Color::BrightRed => "101".to_string(),
            Color::BrightGreen => "102".to_string(),
            Color::BrightYellow => "103".to_string(),
            Color::BrightBlue => "104".to_string(),
            Color::BrightMagenta => "105".to_string(),
            Color::BrightCyan => "106".to_string(),
            Color::BrightWhite => "107".to_string(),
            Color::Rgb(r, g, b) => format!("48;2;{};{};{}", r, g, b),
            Color::Ansi256(c) => format!("48;5;{}", c),
        }
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
}
