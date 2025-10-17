/// Alternative Character Set (ACS) for box drawing and special characters
///
/// These are special characters used for drawing boxes, borders, and other
/// graphical elements in terminal applications.

/// ACS character type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AcsChar(pub char);

impl AcsChar {
    /// Get the character representation
    pub fn as_char(&self) -> char {
        self.0
    }
}

/// Upper left corner
pub const ACS_ULCORNER: AcsChar = AcsChar('┌');

/// Lower left corner
pub const ACS_LLCORNER: AcsChar = AcsChar('└');

/// Upper right corner
pub const ACS_URCORNER: AcsChar = AcsChar('┐');

/// Lower right corner
pub const ACS_LRCORNER: AcsChar = AcsChar('┘');

/// Horizontal line
pub const ACS_HLINE: AcsChar = AcsChar('─');

/// Vertical line
pub const ACS_VLINE: AcsChar = AcsChar('│');

/// Left tee (├)
pub const ACS_LTEE: AcsChar = AcsChar('├');

/// Right tee (┤)
pub const ACS_RTEE: AcsChar = AcsChar('┤');

/// Top tee (┬)
pub const ACS_TTEE: AcsChar = AcsChar('┬');

/// Bottom tee (┴)
pub const ACS_BTEE: AcsChar = AcsChar('┴');

/// Plus/crossover (┼)
pub const ACS_PLUS: AcsChar = AcsChar('┼');

/// Diamond (◆)
pub const ACS_DIAMOND: AcsChar = AcsChar('◆');

/// Checker board (░)
pub const ACS_CKBOARD: AcsChar = AcsChar('░');

/// Degree symbol (°)
pub const ACS_DEGREE: AcsChar = AcsChar('°');

/// Plus/minus (±)
pub const ACS_PLMINUS: AcsChar = AcsChar('±');

/// Bullet (•)
pub const ACS_BULLET: AcsChar = AcsChar('•');

/// Arrow pointing left (←)
pub const ACS_LARROW: AcsChar = AcsChar('←');

/// Arrow pointing right (→)
pub const ACS_RARROW: AcsChar = AcsChar('→');

/// Arrow pointing down (↓)
pub const ACS_DARROW: AcsChar = AcsChar('↓');

/// Arrow pointing up (↑)
pub const ACS_UARROW: AcsChar = AcsChar('↑');

/// Board of squares (▒)
pub const ACS_BOARD: AcsChar = AcsChar('▒');

/// Lantern symbol (▓)
pub const ACS_LANTERN: AcsChar = AcsChar('▓');

/// Solid square block (█)
pub const ACS_BLOCK: AcsChar = AcsChar('█');

/// Scan line 1 (⎺)
pub const ACS_S1: AcsChar = AcsChar('⎺');

/// Scan line 3 (⎻)
pub const ACS_S3: AcsChar = AcsChar('⎻');

/// Scan line 7 (⎼)
pub const ACS_S7: AcsChar = AcsChar('⎼');

/// Scan line 9 (⎽)
pub const ACS_S9: AcsChar = AcsChar('⎽');

/// Less than or equal (≤)
pub const ACS_LEQUAL: AcsChar = AcsChar('≤');

/// Greater than or equal (≥)
pub const ACS_GEQUAL: AcsChar = AcsChar('≥');

/// Pi (π)
pub const ACS_PI: AcsChar = AcsChar('π');

/// Not equal (≠)
pub const ACS_NEQUAL: AcsChar = AcsChar('≠');

/// Pound sterling (£)
pub const ACS_STERLING: AcsChar = AcsChar('£');

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_acs_corners() {
        assert_eq!(ACS_ULCORNER.as_char(), '┌');
        assert_eq!(ACS_URCORNER.as_char(), '┐');
        assert_eq!(ACS_LLCORNER.as_char(), '└');
        assert_eq!(ACS_LRCORNER.as_char(), '┘');
    }

    #[test]
    fn test_acs_lines() {
        assert_eq!(ACS_HLINE.as_char(), '─');
        assert_eq!(ACS_VLINE.as_char(), '│');
    }

    #[test]
    fn test_acs_tees() {
        assert_eq!(ACS_LTEE.as_char(), '├');
        assert_eq!(ACS_RTEE.as_char(), '┤');
        assert_eq!(ACS_TTEE.as_char(), '┬');
        assert_eq!(ACS_BTEE.as_char(), '┴');
        assert_eq!(ACS_PLUS.as_char(), '┼');
    }

    #[test]
    fn test_acs_symbols() {
        assert_eq!(ACS_DIAMOND.as_char(), '◆');
        assert_eq!(ACS_BULLET.as_char(), '•');
        assert_eq!(ACS_BLOCK.as_char(), '█');
    }

    #[test]
    fn test_acs_arrows() {
        assert_eq!(ACS_LARROW.as_char(), '←');
        assert_eq!(ACS_RARROW.as_char(), '→');
        assert_eq!(ACS_UARROW.as_char(), '↑');
        assert_eq!(ACS_DARROW.as_char(), '↓');
    }

    #[test]
    fn test_acs_math() {
        assert_eq!(ACS_LEQUAL.as_char(), '≤');
        assert_eq!(ACS_GEQUAL.as_char(), '≥');
        assert_eq!(ACS_PI.as_char(), 'π');
        assert_eq!(ACS_PLMINUS.as_char(), '±');
    }

    #[test]
    fn test_acs_equality() {
        assert_eq!(ACS_ULCORNER, ACS_ULCORNER);
        assert_ne!(ACS_ULCORNER, ACS_URCORNER);
    }

    #[test]
    fn test_acs_char_clone() {
        let ch1 = ACS_DIAMOND;
        let ch2 = ch1;
        assert_eq!(ch1, ch2);
    }
}
