use crate::cell::Cell;

/// Represents a dirty region within a line
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DirtyRegion {
    /// First changed column (inclusive), None if line is clean
    pub first_changed: Option<u16>,
    /// Last changed column (inclusive), None if line is clean
    pub last_changed: Option<u16>,
}

impl DirtyRegion {
    /// Create a clean (no changes) dirty region
    pub fn clean() -> Self {
        Self {
            first_changed: None,
            last_changed: None,
        }
    }

    /// Create a dirty region covering the entire line
    pub fn full(width: u16) -> Self {
        Self {
            first_changed: Some(0),
            last_changed: Some(width.saturating_sub(1)),
        }
    }

    /// Mark a range as dirty
    pub fn mark(&mut self, start: u16, end: u16) {
        match (self.first_changed, self.last_changed) {
            (None, None) => {
                // First time marking this line
                self.first_changed = Some(start);
                self.last_changed = Some(end);
            }
            (Some(first), Some(last)) => {
                // Expand the dirty region
                self.first_changed = Some(first.min(start));
                self.last_changed = Some(last.max(end));
            }
            _ => unreachable!("Invalid dirty region state"),
        }
    }

    /// Check if the region is dirty
    pub fn is_dirty(&self) -> bool {
        self.first_changed.is_some()
    }

    /// Get the range of changed columns (inclusive), if any
    pub fn range(&self) -> Option<(u16, u16)> {
        match (self.first_changed, self.last_changed) {
            (Some(first), Some(last)) => Some((first, last)),
            _ => None,
        }
    }
}

/// Find the first and last difference in a line
pub fn find_line_diff(old_line: &[Cell], new_line: &[Cell]) -> Option<(usize, usize)> {
    if old_line.len() != new_line.len() {
        // Different lengths - entire line is different
        return Some((0, new_line.len().saturating_sub(1)));
    }

    if old_line.is_empty() {
        return None;
    }

    // Find first difference
    let first_diff = old_line
        .iter()
        .zip(new_line.iter())
        .position(|(old, new)| old != new)?;

    // Find last difference (search from the end)
    let last_diff = old_line
        .iter()
        .zip(new_line.iter())
        .rposition(|(old, new)| old != new)
        .unwrap_or(first_diff);

    Some((first_diff, last_diff))
}

/// Helper function to hash a Color
fn hash_color(color: &crate::color::Color) -> u64 {
    use crate::color::Color;
    match color {
        Color::Black => 0,
        Color::Red => 1,
        Color::Green => 2,
        Color::Yellow => 3,
        Color::Blue => 4,
        Color::Magenta => 5,
        Color::Cyan => 6,
        Color::White => 7,
        Color::BrightBlack => 8,
        Color::BrightRed => 9,
        Color::BrightGreen => 10,
        Color::BrightYellow => 11,
        Color::BrightBlue => 12,
        Color::BrightMagenta => 13,
        Color::BrightCyan => 14,
        Color::BrightWhite => 15,
        Color::Rgb(r, g, b) => {
            // Combine RGB values into a hash
            let mut h = 256u64;
            h = h.wrapping_mul(31).wrapping_add(*r as u64);
            h = h.wrapping_mul(31).wrapping_add(*g as u64);
            h = h.wrapping_mul(31).wrapping_add(*b as u64);
            h
        }
        Color::Ansi256(c) => 512u64.wrapping_add(*c as u64),
    }
}

/// Compute hash for a line (used for line matching)
pub fn hash_line(cells: &[Cell]) -> u64 {
    let mut hash = 0u64;
    for cell in cells {
        // Combine character, attributes, and colors into hash
        hash = hash.wrapping_mul(31).wrapping_add(cell.ch as u64);
        hash = hash.wrapping_mul(31).wrapping_add(cell.attr.bits() as u64);
        hash = hash
            .wrapping_mul(31)
            .wrapping_add(cell.fg.map(|c| hash_color(&c)).unwrap_or(0));
        hash = hash
            .wrapping_mul(31)
            .wrapping_add(cell.bg.map(|c| hash_color(&c)).unwrap_or(0));
    }
    hash
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::attr::Attr;
    use crate::color::Color;

    #[test]
    fn test_dirty_region_clean() {
        let region = DirtyRegion::clean();
        assert!(!region.is_dirty());
        assert_eq!(region.range(), None);
    }

    #[test]
    fn test_dirty_region_full() {
        let region = DirtyRegion::full(80);
        assert!(region.is_dirty());
        assert_eq!(region.range(), Some((0, 79)));
    }

    #[test]
    fn test_dirty_region_mark_first_time() {
        let mut region = DirtyRegion::clean();
        region.mark(10, 20);
        assert!(region.is_dirty());
        assert_eq!(region.range(), Some((10, 20)));
    }

    #[test]
    fn test_dirty_region_mark_expand() {
        let mut region = DirtyRegion::clean();
        region.mark(10, 20);
        region.mark(5, 15);
        assert_eq!(region.range(), Some((5, 20)));
    }

    #[test]
    fn test_dirty_region_mark_expand_both_ends() {
        let mut region = DirtyRegion::clean();
        region.mark(10, 20);
        region.mark(5, 25);
        assert_eq!(region.range(), Some((5, 25)));
    }

    #[test]
    fn test_find_line_diff_identical() {
        let line1 = vec![Cell::new('A'), Cell::new('B'), Cell::new('C')];
        let line2 = vec![Cell::new('A'), Cell::new('B'), Cell::new('C')];
        assert_eq!(find_line_diff(&line1, &line2), None);
    }

    #[test]
    fn test_find_line_diff_single_char() {
        let line1 = vec![Cell::new('A'), Cell::new('B'), Cell::new('C')];
        let line2 = vec![Cell::new('A'), Cell::new('X'), Cell::new('C')];
        assert_eq!(find_line_diff(&line1, &line2), Some((1, 1)));
    }

    #[test]
    fn test_find_line_diff_multiple_chars() {
        let line1 = vec![Cell::new('A'), Cell::new('B'), Cell::new('C')];
        let line2 = vec![Cell::new('X'), Cell::new('Y'), Cell::new('C')];
        assert_eq!(find_line_diff(&line1, &line2), Some((0, 1)));
    }

    #[test]
    fn test_find_line_diff_all_different() {
        let line1 = vec![Cell::new('A'), Cell::new('B'), Cell::new('C')];
        let line2 = vec![Cell::new('X'), Cell::new('Y'), Cell::new('Z')];
        assert_eq!(find_line_diff(&line1, &line2), Some((0, 2)));
    }

    #[test]
    fn test_find_line_diff_different_lengths() {
        let line1 = vec![Cell::new('A'), Cell::new('B')];
        let line2 = vec![Cell::new('A'), Cell::new('B'), Cell::new('C')];
        assert_eq!(find_line_diff(&line1, &line2), Some((0, 2)));
    }

    #[test]
    fn test_find_line_diff_empty() {
        let line1: Vec<Cell> = vec![];
        let line2: Vec<Cell> = vec![];
        assert_eq!(find_line_diff(&line1, &line2), None);
    }

    #[test]
    fn test_find_line_diff_style_change() {
        let line1 = vec![Cell::new('A')];
        let line2 = vec![Cell::with_style('A', Attr::BOLD, None, None)];
        assert_eq!(find_line_diff(&line1, &line2), Some((0, 0)));
    }

    #[test]
    fn test_hash_line_identical() {
        let line1 = vec![Cell::new('A'), Cell::new('B'), Cell::new('C')];
        let line2 = vec![Cell::new('A'), Cell::new('B'), Cell::new('C')];
        assert_eq!(hash_line(&line1), hash_line(&line2));
    }

    #[test]
    fn test_hash_line_different_chars() {
        let line1 = vec![Cell::new('A'), Cell::new('B'), Cell::new('C')];
        let line2 = vec![Cell::new('X'), Cell::new('Y'), Cell::new('Z')];
        assert_ne!(hash_line(&line1), hash_line(&line2));
    }

    #[test]
    fn test_hash_line_different_attrs() {
        let line1 = vec![Cell::new('A')];
        let line2 = vec![Cell::with_style('A', Attr::BOLD, None, None)];
        assert_ne!(hash_line(&line1), hash_line(&line2));
    }

    #[test]
    fn test_hash_line_different_colors() {
        let line1 = vec![Cell::with_style('A', Attr::NORMAL, Some(Color::Red), None)];
        let line2 = vec![Cell::with_style('A', Attr::NORMAL, Some(Color::Blue), None)];
        assert_ne!(hash_line(&line1), hash_line(&line2));
    }

    #[test]
    fn test_hash_line_empty() {
        let line1: Vec<Cell> = vec![];
        let line2: Vec<Cell> = vec![];
        assert_eq!(hash_line(&line1), hash_line(&line2));
        assert_eq!(hash_line(&line1), 0);
    }

    #[test]
    fn test_hash_line_with_spaces() {
        let line1 = vec![Cell::new(' '), Cell::new(' '), Cell::new(' ')];
        let line2 = vec![Cell::new(' '), Cell::new(' '), Cell::new(' ')];
        assert_eq!(hash_line(&line1), hash_line(&line2));
    }
}
