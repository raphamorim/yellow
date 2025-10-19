use crate::attr::Attr;
use crate::color::Color;

/// A single cell in the screen buffer, containing a character and its styling
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cell {
    /// The character to display
    pub ch: char,
    /// Text attributes (bold, underline, etc.)
    pub attr: Attr,
    /// Foreground color
    pub fg: Option<Color>,
    /// Background color
    pub bg: Option<Color>,
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
        Self { ch, attr, fg, bg }
    }

    /// Check if this cell is a blank (space with no styling)
    pub fn is_blank(&self) -> bool {
        self.ch == ' ' && self.attr == Attr::NORMAL && self.fg.is_none() && self.bg.is_none()
    }

    /// Check if this cell has the same styling as another (ignoring character)
    pub fn same_style(&self, other: &Cell) -> bool {
        self.attr == other.attr && self.fg == other.fg && self.bg == other.bg
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
    fn test_cell_new() {
        let cell = Cell::new('A');
        assert_eq!(cell.ch, 'A');
        assert_eq!(cell.attr, Attr::NORMAL);
        assert_eq!(cell.fg, None);
        assert_eq!(cell.bg, None);
    }

    #[test]
    fn test_cell_blank() {
        let cell = Cell::blank();
        assert_eq!(cell.ch, ' ');
        assert_eq!(cell.attr, Attr::NORMAL);
        assert_eq!(cell.fg, None);
        assert_eq!(cell.bg, None);
        assert!(cell.is_blank());
    }

    #[test]
    fn test_cell_with_style() {
        let cell = Cell::with_style('B', Attr::BOLD, Some(Color::Red), Some(Color::Black));
        assert_eq!(cell.ch, 'B');
        assert_eq!(cell.attr, Attr::BOLD);
        assert_eq!(cell.fg, Some(Color::Red));
        assert_eq!(cell.bg, Some(Color::Black));
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
        let cell1 = Cell::with_style('X', Attr::BOLD | Attr::UNDERLINE, Some(Color::Green), Some(Color::Blue));
        let cell2 = cell1.clone();

        assert_eq!(cell1, cell2);
        assert_eq!(cell2.ch, 'X');
        assert_eq!(cell2.attr, Attr::BOLD | Attr::UNDERLINE);
        assert_eq!(cell2.fg, Some(Color::Green));
        assert_eq!(cell2.bg, Some(Color::Blue));
    }
}
