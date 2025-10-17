use std::ops::{BitAnd, BitOr, Not};

/// Text attributes
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Attr(pub(crate) u16);

impl Attr {
    pub const NORMAL: Attr = Attr(0);
    pub const BOLD: Attr = Attr(1 << 0);
    pub const DIM: Attr = Attr(1 << 1);
    pub const ITALIC: Attr = Attr(1 << 2);
    pub const UNDERLINE: Attr = Attr(1 << 3);
    pub const BLINK: Attr = Attr(1 << 4);
    pub const REVERSE: Attr = Attr(1 << 5);
    pub const HIDDEN: Attr = Attr(1 << 6);
    pub const STRIKETHROUGH: Attr = Attr(1 << 7);

    pub const fn new() -> Self {
        Self::NORMAL
    }

    pub const fn bits(&self) -> u16 {
        self.0
    }

    pub const fn contains(&self, other: Attr) -> bool {
        (self.0 & other.0) == other.0
    }

    pub const fn is_empty(&self) -> bool {
        self.0 == 0
    }

    pub(crate) fn to_ansi_codes(&self) -> Vec<&'static str> {
        let mut codes = Vec::new();

        if self.contains(Attr::BOLD) {
            codes.push("1");
        }
        if self.contains(Attr::DIM) {
            codes.push("2");
        }
        if self.contains(Attr::ITALIC) {
            codes.push("3");
        }
        if self.contains(Attr::UNDERLINE) {
            codes.push("4");
        }
        if self.contains(Attr::BLINK) {
            codes.push("5");
        }
        if self.contains(Attr::REVERSE) {
            codes.push("7");
        }
        if self.contains(Attr::HIDDEN) {
            codes.push("8");
        }
        if self.contains(Attr::STRIKETHROUGH) {
            codes.push("9");
        }

        codes
    }
}

impl BitOr for Attr {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Attr(self.0 | rhs.0)
    }
}

impl BitAnd for Attr {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        Attr(self.0 & rhs.0)
    }
}

impl Not for Attr {
    type Output = Self;

    fn not(self) -> Self::Output {
        Attr(!self.0)
    }
}

impl Default for Attr {
    fn default() -> Self {
        Self::NORMAL
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_attr_creation() {
        let attr = Attr::new();
        assert!(attr.is_empty());
        assert_eq!(attr.bits(), 0);
    }

    #[test]
    fn test_attr_bitor() {
        let attr = Attr::BOLD | Attr::UNDERLINE;
        assert!(attr.contains(Attr::BOLD));
        assert!(attr.contains(Attr::UNDERLINE));
        assert!(!attr.contains(Attr::ITALIC));
    }

    #[test]
    fn test_attr_contains() {
        let attr = Attr::BOLD | Attr::ITALIC;
        assert!(attr.contains(Attr::BOLD));
        assert!(attr.contains(Attr::ITALIC));
        assert!(attr.contains(Attr::BOLD | Attr::ITALIC));
        assert!(!attr.contains(Attr::UNDERLINE));
    }

    #[test]
    fn test_attr_ansi_codes() {
        let attr = Attr::BOLD | Attr::UNDERLINE;
        let codes = attr.to_ansi_codes();
        assert!(codes.contains(&"1"));
        assert!(codes.contains(&"4"));
        assert_eq!(codes.len(), 2);
    }

    #[test]
    fn test_attr_normal() {
        let attr = Attr::NORMAL;
        assert!(attr.is_empty());
        assert_eq!(attr.to_ansi_codes().len(), 0);
    }

    #[test]
    fn test_attr_all() {
        let attr = Attr::BOLD
            | Attr::DIM
            | Attr::ITALIC
            | Attr::UNDERLINE
            | Attr::BLINK
            | Attr::REVERSE
            | Attr::HIDDEN
            | Attr::STRIKETHROUGH;
        let codes = attr.to_ansi_codes();
        assert_eq!(codes.len(), 8);
    }

    #[test]
    fn test_attr_equality() {
        assert_eq!(Attr::BOLD, Attr::BOLD);
        assert_ne!(Attr::BOLD, Attr::ITALIC);
        assert_eq!(Attr::BOLD | Attr::ITALIC, Attr::ITALIC | Attr::BOLD);
    }
}
