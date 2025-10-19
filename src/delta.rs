use crate::cell::Cell;
use crate::Color;

/// Represents a dirty region within a line
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DirtyRegion {
    /// First changed column (inclusive), None if line is clean
    pub first_changed: Option<u16>,
    /// Last changed column (inclusive), None if line is clean
    pub last_changed: Option<u16>,
}

/// Represents a scroll operation (like ncurses' scroll hunks)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScrollOp {
    /// Starting line of the scroll region
    pub start: usize,
    /// Number of lines in the scroll region
    pub size: usize,
    /// Number of lines to shift (positive = scroll up, negative = scroll down)
    pub shift: isize,
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
///
/// Optimized with early exit and chunk-based comparison for better performance.
pub fn find_line_diff(old_line: &[Cell], new_line: &[Cell]) -> Option<(usize, usize)> {
    let len = old_line.len();

    if len != new_line.len() {
        // Different lengths - entire line is different
        return Some((0, new_line.len().saturating_sub(1)));
    }

    if len == 0 {
        return None;
    }

    // Fast path: check if lines are identical using memory comparison
    // This is much faster than cell-by-cell comparison for identical lines
    if old_line == new_line {
        return None;
    }

    // Find first difference - scan forward
    let mut first_diff = 0;
    while first_diff < len && old_line[first_diff] == new_line[first_diff] {
        first_diff += 1;
    }

    // If we reached the end, lines are identical (shouldn't happen due to fast path)
    if first_diff == len {
        return None;
    }

    // Find last difference - scan backward from end
    let mut last_diff = len - 1;
    while last_diff > first_diff && old_line[last_diff] == new_line[last_diff] {
        last_diff -= 1;
    }

    Some((first_diff, last_diff))
}

/// Compute hash for a line (used for line matching)
///
/// Optimized version using FNV-1a hash algorithm which is faster than
/// polynomial rolling hash and provides good distribution.
pub fn hash_line(cells: &[Cell]) -> u64 {
    // Empty lines hash to 0 (for compatibility with blank line detection)
    if cells.is_empty() {
        return 0;
    }

    // FNV-1a constants
    const FNV_OFFSET_BASIS: u64 = 0xcbf29ce484222325;
    const FNV_PRIME: u64 = 0x100000001b3;

    let mut hash = FNV_OFFSET_BASIS;

    for cell in cells {
        // Hash character (4 bytes)
        let ch_bytes = (cell.ch as u32).to_ne_bytes();
        for &byte in &ch_bytes {
            hash ^= byte as u64;
            hash = hash.wrapping_mul(FNV_PRIME);
        }

        // Hash attributes (2 bytes)
        let attr_bytes = cell.attr.bits().to_ne_bytes();
        for &byte in &attr_bytes {
            hash ^= byte as u64;
            hash = hash.wrapping_mul(FNV_PRIME);
        }

        // Hash colors safely by hashing their discriminant and data
        fn hash_color(hash: &mut u64, color: Option<Color>) {
            const FNV_PRIME: u64 = 0x100000001b3;

            match color {
                None => {
                    *hash ^= 0;
                    *hash = hash.wrapping_mul(FNV_PRIME);
                }
                Some(Color::Black) => { *hash ^= 1; *hash = hash.wrapping_mul(FNV_PRIME); }
                Some(Color::Red) => { *hash ^= 2; *hash = hash.wrapping_mul(FNV_PRIME); }
                Some(Color::Green) => { *hash ^= 3; *hash = hash.wrapping_mul(FNV_PRIME); }
                Some(Color::Yellow) => { *hash ^= 4; *hash = hash.wrapping_mul(FNV_PRIME); }
                Some(Color::Blue) => { *hash ^= 5; *hash = hash.wrapping_mul(FNV_PRIME); }
                Some(Color::Magenta) => { *hash ^= 6; *hash = hash.wrapping_mul(FNV_PRIME); }
                Some(Color::Cyan) => { *hash ^= 7; *hash = hash.wrapping_mul(FNV_PRIME); }
                Some(Color::White) => { *hash ^= 8; *hash = hash.wrapping_mul(FNV_PRIME); }
                Some(Color::BrightBlack) => { *hash ^= 9; *hash = hash.wrapping_mul(FNV_PRIME); }
                Some(Color::BrightRed) => { *hash ^= 10; *hash = hash.wrapping_mul(FNV_PRIME); }
                Some(Color::BrightGreen) => { *hash ^= 11; *hash = hash.wrapping_mul(FNV_PRIME); }
                Some(Color::BrightYellow) => { *hash ^= 12; *hash = hash.wrapping_mul(FNV_PRIME); }
                Some(Color::BrightBlue) => { *hash ^= 13; *hash = hash.wrapping_mul(FNV_PRIME); }
                Some(Color::BrightMagenta) => { *hash ^= 14; *hash = hash.wrapping_mul(FNV_PRIME); }
                Some(Color::BrightCyan) => { *hash ^= 15; *hash = hash.wrapping_mul(FNV_PRIME); }
                Some(Color::BrightWhite) => { *hash ^= 16; *hash = hash.wrapping_mul(FNV_PRIME); }
                Some(Color::Ansi256(c)) => {
                    *hash ^= 17;
                    *hash = hash.wrapping_mul(FNV_PRIME);
                    *hash ^= c as u64;
                    *hash = hash.wrapping_mul(FNV_PRIME);
                }
                Some(Color::Rgb(r, g, b)) => {
                    *hash ^= 18;
                    *hash = hash.wrapping_mul(FNV_PRIME);
                    *hash ^= r as u64;
                    *hash = hash.wrapping_mul(FNV_PRIME);
                    *hash ^= g as u64;
                    *hash = hash.wrapping_mul(FNV_PRIME);
                    *hash ^= b as u64;
                    *hash = hash.wrapping_mul(FNV_PRIME);
                }
                Some(Color::Reset) => { *hash ^= 19; *hash = hash.wrapping_mul(FNV_PRIME); }
            }
        }

        hash_color(&mut hash, cell.fg());
        hash_color(&mut hash, cell.bg());
    }

    hash
}

/// Detect scroll operations using hash-based line matching (Modified Heckel's Algorithm)
/// Inspired by ncurses hashmap.c
pub fn detect_scrolls(
    old_hashes: &[u64],
    new_hashes: &[u64],
) -> Vec<ScrollOp> {
    let old_len = old_hashes.len();
    let new_len = new_hashes.len();

    if old_len == 0 || new_len == 0 {
        return vec![];
    }

    // Build mapping: new_line_index -> old_line_index
    let mut old_num: Vec<Option<usize>> = vec![None; new_len];

    // Step 1: Find unique matches (hash appears exactly once in both old and new)
    for new_i in 0..new_len {
        let hash = new_hashes[new_i];
        if hash == 0 {
            continue; // Skip blank lines
        }

        // Count occurrences in new
        let new_count = new_hashes.iter().filter(|&&h| h == hash).count();
        if new_count != 1 {
            continue; // Not unique in new
        }

        // Find in old
        let old_matches: Vec<usize> = old_hashes
            .iter()
            .enumerate()
            .filter(|(_, h)| **h == hash)
            .map(|(i, _)| i)
            .collect();

        if old_matches.len() == 1 {
            // Unique match found
            old_num[new_i] = Some(old_matches[0]);
        }
    }

    // Step 2: Grow matches forward and backward
    // If line N matched and N+1 also matches, extend the hunk
    for new_i in 0..new_len {
        if let Some(old_i) = old_num[new_i] {
            // Try to extend forward
            let mut offset = 1;
            while new_i + offset < new_len
                && old_i + offset < old_len
                && old_num[new_i + offset].is_none()
                && new_hashes[new_i + offset] == old_hashes[old_i + offset]
                && new_hashes[new_i + offset] != 0
            {
                old_num[new_i + offset] = Some(old_i + offset);
                offset += 1;
            }

            // Try to extend backward
            offset = 1;
            while new_i >= offset
                && old_i >= offset
                && old_num[new_i - offset].is_none()
                && new_hashes[new_i - offset] == old_hashes[old_i - offset]
                && new_hashes[new_i - offset] != 0
            {
                old_num[new_i - offset] = Some(old_i - offset);
                offset += 1;
            }
        }
    }

    // Step 3: Find scroll hunks (contiguous regions with same shift)
    let mut scrolls = Vec::new();
    let mut i = 0;

    while i < new_len {
        if let Some(old_i) = old_num[i] {
            let shift = old_i as isize - i as isize;

            // Find contiguous region with same shift
            let start = i;
            let mut end = i;

            while end + 1 < new_len {
                if let Some(next_old) = old_num[end + 1] {
                    let next_shift = next_old as isize - (end + 1) as isize;
                    if next_shift == shift {
                        end += 1;
                    } else {
                        break;
                    }
                } else {
                    break;
                }
            }

            let size = end - start + 1;

            // Apply heuristics (from ncurses):
            // - Minimum hunk size of 3 lines
            // - Accept if efficient enough: size + min(size/8, 2) >= abs(shift)
            let min_efficiency = size + (size / 8).min(2);
            let shift_abs = shift.unsigned_abs();

            if size >= 3 && min_efficiency >= shift_abs {
                scrolls.push(ScrollOp {
                    start,
                    size,
                    shift,
                });
            }

            i = end + 1;
        } else {
            i += 1;
        }
    }

    scrolls
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

    #[test]
    fn test_detect_scrolls_empty() {
        let old: Vec<u64> = vec![];
        let new: Vec<u64> = vec![];
        let scrolls = detect_scrolls(&old, &new);
        assert_eq!(scrolls.len(), 0);
    }

    #[test]
    fn test_detect_scrolls_no_match() {
        // All different hashes - no scrolling detected
        let old = vec![1, 2, 3, 4, 5];
        let new = vec![6, 7, 8, 9, 10];
        let scrolls = detect_scrolls(&old, &new);
        assert_eq!(scrolls.len(), 0);
    }

    #[test]
    fn test_detect_scrolls_scroll_up() {
        // Lines 0-2 deleted, lines 3-7 moved up by 3
        // Old: [1, 2, 3, A, B, C, D, E]
        // New: [A, B, C, D, E, 4, 5, 6]
        let old = vec![1, 2, 3, 100, 101, 102, 103, 104];
        let new = vec![100, 101, 102, 103, 104, 4, 5, 6];
        let scrolls = detect_scrolls(&old, &new);

        assert_eq!(scrolls.len(), 1);
        assert_eq!(scrolls[0].start, 0); // Lines now at position 0
        assert_eq!(scrolls[0].size, 5); // 5 lines moved
        assert_eq!(scrolls[0].shift, 3); // Moved up by 3 (from index 3 to 0)
    }

    #[test]
    fn test_detect_scrolls_scroll_down() {
        // Lines inserted at top, existing lines moved down
        // Old: [A, B, C, D, E]
        // New: [1, 2, 3, A, B, C, D, E]
        let old = vec![100, 101, 102, 103, 104];
        let new = vec![1, 2, 3, 100, 101, 102, 103, 104];
        let scrolls = detect_scrolls(&old, &new);

        assert_eq!(scrolls.len(), 1);
        assert_eq!(scrolls[0].start, 3); // Lines now at position 3
        assert_eq!(scrolls[0].size, 5); // 5 lines moved
        assert_eq!(scrolls[0].shift, -3); // Moved down by 3 (from index 0 to 3)
    }

    #[test]
    fn test_detect_scrolls_too_small_hunk() {
        // Only 2 lines match - below minimum hunk size of 3
        let old = vec![1, 100, 101, 2];
        let new = vec![100, 101, 3, 4];
        let scrolls = detect_scrolls(&old, &new);

        // Should not detect scroll (hunk too small)
        assert_eq!(scrolls.len(), 0);
    }

    #[test]
    fn test_detect_scrolls_minimum_hunk_size() {
        // Exactly 3 lines match - minimum hunk size
        let old = vec![1, 100, 101, 102, 2];
        let new = vec![100, 101, 102, 3, 4];
        let scrolls = detect_scrolls(&old, &new);

        assert_eq!(scrolls.len(), 1);
        assert_eq!(scrolls[0].size, 3);
    }

    #[test]
    fn test_detect_scrolls_ignore_blank_lines() {
        // Blank lines (hash=0) should not be matched
        let old = vec![0, 0, 100, 101, 102];
        let new = vec![100, 101, 102, 0, 0];
        let scrolls = detect_scrolls(&old, &new);

        assert_eq!(scrolls.len(), 1);
        assert_eq!(scrolls[0].start, 0);
        assert_eq!(scrolls[0].size, 3);
        assert_eq!(scrolls[0].shift, 2); // Moved from index 2 to 0
    }

    #[test]
    fn test_detect_scrolls_duplicate_hashes() {
        // Duplicate hashes should not be matched (not unique)
        let old = vec![100, 100, 101, 102];
        let new = vec![101, 102, 100, 100];
        let scrolls = detect_scrolls(&old, &new);

        // Only 101, 102 should match (unique hashes)
        // But 2 lines is below minimum, so no scroll detected
        assert_eq!(scrolls.len(), 0);
    }

    #[test]
    fn test_detect_scrolls_grow_matches() {
        // Should extend matches forward/backward
        // Old: [1, A, B, C, D, 2]
        // New: [A, B, C, D, 3, 4]
        // Even if only A or C is unique, should match all A,B,C,D
        let old = vec![1, 100, 101, 102, 103, 2];
        let new = vec![100, 101, 102, 103, 3, 4];
        let scrolls = detect_scrolls(&old, &new);

        assert_eq!(scrolls.len(), 1);
        assert_eq!(scrolls[0].size, 4); // All 4 lines matched
    }

    #[test]
    fn test_detect_scrolls_efficiency_heuristic() {
        // Heuristic: size + min(size/8, 2) >= abs(shift)
        // Large shift with small hunk should be rejected
        // Use unique values to avoid accidental matches
        let old = vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 100, 101, 102];
        let new = vec![100, 101, 102, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
        let scrolls = detect_scrolls(&old, &new);

        // shift = 10 (from position 10 to 0), size = 3
        // min_efficiency = 3 + min(3/8, 2) = 3 + 0 = 3
        // 3 < 10, so should be rejected
        assert_eq!(scrolls.len(), 0);
    }

    #[test]
    fn test_detect_scrolls_multiple_hunks() {
        // Multiple independent scroll regions
        // Make hunks large enough to pass efficiency heuristic
        // size=8, shift=7: min_eff = 8+min(8/8,2) = 8+1 = 9 >= 7 ✓
        // Old: [A,B,C,D,E,F,G,H, 0, X,Y,Z,W,V,U,T,S]
        // New: [X,Y,Z,W,V,U,T,S, 0, A,B,C,D,E,F,G,H]
        let old = vec![100, 101, 102, 103, 104, 105, 106, 107, 0, 200, 201, 202, 203, 204, 205, 206, 207];
        let new = vec![200, 201, 202, 203, 204, 205, 206, 207, 0, 100, 101, 102, 103, 104, 105, 106, 107];
        let scrolls = detect_scrolls(&old, &new);

        // Should detect 2 separate scroll hunks
        assert_eq!(scrolls.len(), 2);

        // First hunk: moved from position 9 to 0 (shift = 9)
        assert_eq!(scrolls[0].start, 0);
        assert_eq!(scrolls[0].size, 8);
        assert_eq!(scrolls[0].shift, 9);

        // Second hunk: moved from position 0 to 9 (shift = -9)
        assert_eq!(scrolls[1].start, 9);
        assert_eq!(scrolls[1].size, 8);
        assert_eq!(scrolls[1].shift, -9);
    }
}
