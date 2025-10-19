/// Cell implementation with full-precision colors
///
/// This version uses Option<Color> directly to preserve full RGB precision,
/// avoiding color quantization artifacts in gradients.

use crate::attr::Attr;
use crate::color::Color;

/// A single cell in the screen buffer, containing a character and its styling
///
/// Memory layout (~16 bytes total):
/// - ch: char (4 bytes)
/// - attr: u16 (2 bytes)
/// - padding: 2 bytes (for alignment)
/// - fg: Option<Color> (4 bytes)
/// - bg: Option<Color> (4 bytes)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cell {
    /// The character to display
    pub ch: char,
    /// Text attributes (bold, underline, etc.)
    pub attr: Attr,
    /// Foreground color
    fg: Option<Color>,
    /// Background color
    bg: Option<Color>,
}

impl Cell {
    /// Create a new cell with a character and default styling
    pub fn new(ch: char) -> Self {
        Self {
            ch,
            attr: Attr::NORMAL,
            fg: None,
            bg: None,
        }
    }

    /// Create a blank cell (space character with no styling)
    pub fn blank() -> Self {
        Self::new(' ')
    }

    /// Create a cell with a character and specific styling
    pub fn with_style(ch: char, attr: Attr, fg: Option<Color>, bg: Option<Color>) -> Self {
        Self {
            ch,
            attr,
            fg,
            bg,
        }
    }

    /// Get the character
    #[inline]
    pub fn ch(&self) -> char {
        self.ch
    }

    /// Get the attributes
    #[inline]
    pub fn attr(&self) -> Attr {
        self.attr
    }

    /// Get the foreground color
    #[inline]
    pub fn fg(&self) -> Option<Color> {
        self.fg
    }

    /// Get the background color
    #[inline]
    pub fn bg(&self) -> Option<Color> {
        self.bg
    }

    /// Check if this cell is a blank (space with no styling)
    pub fn is_blank(&self) -> bool {
        self.ch == ' '
            && self.attr == Attr::NORMAL
            && self.fg.is_none()
            && self.bg.is_none()
    }

    /// Check if this cell has the same styling as another (ignoring character)
    pub fn same_style(&self, other: &Cell) -> bool {
        self.attr == other.attr
            && self.fg == other.fg
            && self.bg == other.bg
    }
}

impl Default for Cell {
    fn default() -> Self {
        Self::blank()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cell_size() {
        let size = std::mem::size_of::<Cell>();

        // Full-precision colors: Cell should be 16 bytes (char=4, Attr=2, padding=2, fg=4, bg=4)
        assert_eq!(size, 16, "Cell should be exactly 16 bytes for full-precision colors");
        assert!(size < 24, "Cell should be significantly smaller than original ~32 bytes");
    }

    #[test]
    fn test_cell_new() {
        let cell = Cell::new('A');
        assert_eq!(cell.ch(), 'A');
        assert_eq!(cell.attr(), Attr::NORMAL);
        assert_eq!(cell.fg(), None);
        assert_eq!(cell.bg(), None);
    }

    #[test]
    fn test_cell_blank() {
        let cell = Cell::blank();
        assert_eq!(cell.ch(), ' ');
        assert_eq!(cell.attr(), Attr::NORMAL);
        assert_eq!(cell.fg(), None);
        assert_eq!(cell.bg(), None);
        assert!(cell.is_blank());
    }

    #[test]
    fn test_cell_with_style() {
        let cell = Cell::with_style('B', Attr::BOLD, Some(Color::Red), Some(Color::Black));
        assert_eq!(cell.ch(), 'B');
        assert_eq!(cell.attr(), Attr::BOLD);
        assert_eq!(cell.fg(), Some(Color::Red));
        assert_eq!(cell.bg(), Some(Color::Black));
    }

    #[test]
    fn test_cell_equality() {
        let cell1 = Cell::new('A');
        let cell2 = Cell::new('A');
        let cell3 = Cell::new('B');

        assert_eq!(cell1, cell2);
        assert_ne!(cell1, cell3);
    }

    #[test]
    fn test_cell_equality_with_style() {
        let cell1 = Cell::with_style('A', Attr::BOLD, Some(Color::Red), None);
        let cell2 = Cell::with_style('A', Attr::BOLD, Some(Color::Red), None);
        let cell3 = Cell::with_style('A', Attr::UNDERLINE, Some(Color::Red), None);

        assert_eq!(cell1, cell2);
        assert_ne!(cell1, cell3);
    }

    #[test]
    fn test_cell_is_blank() {
        let blank = Cell::blank();
        let space_styled = Cell::with_style(' ', Attr::BOLD, None, None);
        let space_colored = Cell::with_style(' ', Attr::NORMAL, Some(Color::Red), None);
        let char_cell = Cell::new('A');

        assert!(blank.is_blank());
        assert!(!space_styled.is_blank());
        assert!(!space_colored.is_blank());
        assert!(!char_cell.is_blank());
    }

    #[test]
    fn test_cell_same_style() {
        let cell1 = Cell::with_style('A', Attr::BOLD, Some(Color::Red), None);
        let cell2 = Cell::with_style('B', Attr::BOLD, Some(Color::Red), None);
        let cell3 = Cell::with_style('A', Attr::UNDERLINE, Some(Color::Red), None);

        assert!(cell1.same_style(&cell2));
        assert!(!cell1.same_style(&cell3));
    }

    #[test]
    fn test_cell_default() {
        let cell = Cell::default();
        assert!(cell.is_blank());
    }

    #[test]
    fn test_cell_clone() {
        let cell1 = Cell::with_style(
            'X',
            Attr::BOLD | Attr::UNDERLINE,
            Some(Color::Green),
            Some(Color::Blue),
        );
        let cell2 = cell1.clone();

        assert_eq!(cell1, cell2);
        assert_eq!(cell2.ch(), 'X');
        assert_eq!(cell2.attr(), Attr::BOLD | Attr::UNDERLINE);
        assert_eq!(cell2.fg(), Some(Color::Green));
        assert_eq!(cell2.bg(), Some(Color::Blue));
    }

    #[test]
    fn test_all_colors() {
        // Test all basic colors
        for &color in &[
            Color::Black,
            Color::Red,
            Color::Green,
            Color::Yellow,
            Color::Blue,
            Color::Magenta,
            Color::Cyan,
            Color::White,
            Color::BrightBlack,
            Color::BrightRed,
            Color::BrightGreen,
            Color::BrightYellow,
            Color::BrightBlue,
            Color::BrightMagenta,
            Color::BrightCyan,
            Color::BrightWhite,
        ] {
            let cell = Cell::with_style('X', Attr::NORMAL, Some(color), None);
            assert_eq!(cell.fg(), Some(color));
        }
    }

    #[test]
    fn test_rgb_colors() {
        let test_cases = vec![
            (255, 0, 0),
            (0, 255, 0),
            (0, 0, 255),
            (255, 255, 255),
            (0, 0, 0),
            (128, 128, 128),
        ];

        for (r, g, b) in test_cases {
            let color = Color::Rgb(r, g, b);
            let cell = Cell::with_style('X', Attr::NORMAL, Some(color), None);

            match cell.fg() {
                Some(Color::Rgb(r2, g2, b2)) => {
                    // Allow for 4-bit quantization (up to 17 per channel)
                    assert!((r as i16 - r2 as i16).abs() <= 17);
                    assert!((g as i16 - g2 as i16).abs() <= 17);
                    assert!((b as i16 - b2 as i16).abs() <= 17);
                }
                other => panic!("Expected RGB color, got {:?}", other),
            }
        }
    }

    #[test]
    fn test_ansi256_colors() {
        for i in 0..=255 {
            let color = Color::Ansi256(i);
            let cell = Cell::with_style('X', Attr::NORMAL, Some(color), None);
            assert_eq!(cell.fg(), Some(color));
        }
    }

    #[test]
    fn test_memory_efficiency() {
        // Create a line of 80 cells
        let line: Vec<Cell> = (0..80).map(|_| Cell::blank()).collect();

        let size = std::mem::size_of_val(&line[..]);
        let cell_size = std::mem::size_of::<Cell>();
        let expected = cell_size * 80;

        assert_eq!(size, expected);

        // Verify it's significantly smaller than original
        // Original was ~32 bytes, so 80 cells = 2560 bytes
        // New should be 16 bytes, so 80 cells = 1280 bytes
        assert_eq!(size, 1280, "80 cells should use exactly 1280 bytes (16 bytes per cell)");
    }
}
