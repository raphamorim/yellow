use crate::attr::Attr;
use crate::color::Color;
use crate::error::{Error, Result};
use smallvec::SmallVec;
use std::fmt::Write;
use std::io;

/// A window (subregion of the screen)
pub struct Window {
    height: u16,
    width: u16,
    begin_y: u16,
    begin_x: u16,
    cursor_x: u16,
    cursor_y: u16,
    current_attr: Attr,
    current_fg: Color,
    current_bg: Color,
    buffer: String,
    scroll_enabled: bool,
    // Performance optimization: track last emitted style to avoid redundant codes
    last_emitted_attr: Attr,
    last_emitted_fg: Color,
    last_emitted_bg: Color,
    // Performance optimization: SmallVec for style sequence (stack-allocated for <64 bytes)
    style_sequence_buf: SmallVec<[u8; 64]>,
}

impl Window {
    pub(crate) fn new(height: u16, width: u16, y: u16, x: u16) -> Result<Self> {
        // Performance optimization: pre-allocate buffer based on window size
        // Estimate: ~10 bytes per cell (ANSI codes + character)
        let estimated_capacity = (height as usize * width as usize * 10).min(65536); // Cap at 64KB

        Ok(Self {
            height,
            width,
            begin_y: y,
            begin_x: x,
            cursor_x: 0,
            cursor_y: 0,
            current_attr: Attr::NORMAL,
            current_fg: Color::Reset,
            current_bg: Color::Reset,
            buffer: String::with_capacity(estimated_capacity),
            scroll_enabled: false,
            last_emitted_attr: Attr::NORMAL,
            last_emitted_fg: Color::Reset,
            last_emitted_bg: Color::Reset,
            style_sequence_buf: SmallVec::new(), // Stack-allocated for sequences <64 bytes
        })
    }

    /// Get window dimensions (height, width)
    pub fn get_size(&self) -> (u16, u16) {
        (self.height, self.width)
    }

    /// Get window position (y, x)
    pub fn get_position(&self) -> (u16, u16) {
        (self.begin_y, self.begin_x)
    }

    /// Move cursor within window (relative to window origin)
    pub fn move_cursor(&mut self, y: u16, x: u16) -> Result<()> {
        if y >= self.height || x >= self.width {
            return Err(Error::InvalidCoordinates { y, x });
        }

        // Performance optimization: use relative cursor movement for short distances
        let dy = (y as i32 - self.cursor_y as i32).abs();
        let dx = (x as i32 - self.cursor_x as i32).abs();

        let abs_y = self.begin_y + y;
        let abs_x = self.begin_x + x;

        // Threshold: use relative movement if distance < 4 cells
        if dy == 0 && dx > 0 && dx < 4 {
            // Horizontal movement only
            if x > self.cursor_x {
                write!(self.buffer, "\x1b[{}C", dx)?; // CUF - Cursor Forward
            } else {
                write!(self.buffer, "\x1b[{}D", dx)?; // CUB - Cursor Back
            }
        } else if dx == 0 && dy > 0 && dy < 4 {
            // Vertical movement only
            if y > self.cursor_y {
                write!(self.buffer, "\x1b[{}B", dy)?; // CUD - Cursor Down
            } else {
                write!(self.buffer, "\x1b[{}A", dy)?; // CUU - Cursor Up
            }
        } else {
            // Use absolute positioning for long distances or diagonal movement
            write!(self.buffer, "\x1b[{};{}H", abs_y + 1, abs_x + 1)?; // CUP - Cursor Position
        }

        self.cursor_y = y;
        self.cursor_x = x;
        Ok(())
    }

    /// Print text at current cursor position
    pub fn print(&mut self, text: &str) -> Result<()> {
        // Truncate text if it exceeds window width
        let remaining = (self.width - self.cursor_x) as usize;
        let text_to_print = if text.len() > remaining {
            &text[..remaining]
        } else {
            text
        };

        // Performance optimization: use ECH (Erase Character) for long blank runs
        if text_to_print.len() >= 8 && text_to_print.chars().all(|c| c == ' ') {
            // Use ECH sequence for efficiency
            write!(self.buffer, "\x1b[{}X", text_to_print.len())?;
            self.cursor_x += text_to_print.len() as u16;
            return Ok(());
        }

        self.apply_style()?;
        write!(self.buffer, "{}", text_to_print)?;
        self.cursor_x += text_to_print.len() as u16;
        Ok(())
    }

    /// Move cursor and print
    pub fn mvprint(&mut self, y: u16, x: u16, text: &str) -> Result<()> {
        self.move_cursor(y, x)?;
        self.print(text)
    }

    /// Add a single character
    pub fn addch(&mut self, ch: char) -> Result<()> {
        if self.cursor_x >= self.width {
            return Ok(());
        }

        self.apply_style()?;
        write!(self.buffer, "{}", ch)?;
        self.cursor_x += 1;
        Ok(())
    }

    /// Move cursor and add character
    pub fn mvaddch(&mut self, y: u16, x: u16, ch: char) -> Result<()> {
        self.move_cursor(y, x)?;
        self.addch(ch)
    }

    /// Turn on attributes
    pub fn attron(&mut self, attr: Attr) -> Result<()> {
        self.current_attr = self.current_attr | attr;
        Ok(())
    }

    /// Turn off attributes
    pub fn attroff(&mut self, attr: Attr) -> Result<()> {
        self.current_attr = self.current_attr & !attr;
        Ok(())
    }

    /// Set foreground color
    pub fn set_fg(&mut self, color: Color) -> Result<()> {
        self.current_fg = color;
        Ok(())
    }

    /// Set background color
    pub fn set_bg(&mut self, color: Color) -> Result<()> {
        self.current_bg = color;
        Ok(())
    }

    /// Clear the window
    pub fn clear(&mut self) -> Result<()> {
        // Performance optimization: use ED (Erase in Display) instead of line-by-line clear
        self.move_cursor(0, 0)?;

        // Fill the entire window with blanks using optimized sequences
        for y in 0..self.height {
            if y > 0 {
                self.move_cursor(y, 0)?;
            }
            // Use EL (Erase in Line) to clear to end of line
            write!(self.buffer, "\x1b[K")?;
        }

        self.move_cursor(0, 0)?;
        Ok(())
    }

    /// Draw a border around the window
    pub fn border(
        &mut self,
        ls: char,
        rs: char,
        ts: char,
        bs: char,
        tl: char,
        tr: char,
        bl: char,
        br: char,
    ) -> Result<()> {
        // Top border
        self.mvaddch(0, 0, tl)?;
        for _ in 1..self.width - 1 {
            self.addch(ts)?;
        }
        self.addch(tr)?;

        // Sides
        for y in 1..self.height - 1 {
            self.mvaddch(y, 0, ls)?;
            self.mvaddch(y, self.width - 1, rs)?;
        }

        // Bottom border
        self.mvaddch(self.height - 1, 0, bl)?;
        for _ in 1..self.width - 1 {
            self.addch(bs)?;
        }
        self.addch(br)?;

        Ok(())
    }

    /// Draw a box using ACS line-drawing characters
    pub fn draw_box(&mut self) -> Result<()> {
        use crate::acs::*;
        self.border(
            ACS_VLINE.as_char(),
            ACS_VLINE.as_char(),
            ACS_HLINE.as_char(),
            ACS_HLINE.as_char(),
            ACS_ULCORNER.as_char(),
            ACS_URCORNER.as_char(),
            ACS_LLCORNER.as_char(),
            ACS_LRCORNER.as_char(),
        )
    }

    /// Refresh the window (flush buffer to stdout)
    pub fn refresh(&mut self) -> Result<()> {
        use std::io::Write as IoWrite;
        io::stdout().write_all(self.buffer.as_bytes())?;
        io::stdout().flush()?;
        self.buffer.clear();
        Ok(())
    }

    /// Update internal buffer without refreshing screen
    pub fn wnoutrefresh(&mut self) -> Result<()> {
        use crate::backend::Backend;
        Backend::add_to_update_buffer(&self.buffer)?;
        self.buffer.clear();
        Ok(())
    }

    /// Enable or disable scrolling
    pub fn scrollok(&mut self, enabled: bool) -> Result<()> {
        self.scroll_enabled = enabled;
        Ok(())
    }

    /// Scroll the window up by n lines
    pub fn scroll(&mut self, lines: i16) -> Result<()> {
        if !self.scroll_enabled {
            return Ok(());
        }

        if lines > 0 {
            // Scroll up
            for _ in 0..lines {
                write!(
                    self.buffer,
                    "\x1b[{};{}r",
                    self.begin_y + 1,
                    self.begin_y + self.height
                )?;
                write!(self.buffer, "\x1b[{}H\n", self.begin_y + self.height)?;
                write!(self.buffer, "\x1b[r")?;
            }
        } else if lines < 0 {
            // Scroll down
            for _ in 0..(-lines) {
                write!(
                    self.buffer,
                    "\x1b[{};{}r",
                    self.begin_y + 1,
                    self.begin_y + self.height
                )?;
                write!(self.buffer, "\x1b[{}H\x1bM", self.begin_y + 1)?;
                write!(self.buffer, "\x1b[r")?;
            }
        }

        Ok(())
    }

    fn apply_style(&mut self) -> Result<()> {
        // Performance optimization: only emit ANSI codes if style changed since last emission
        let style_changed = self.current_attr != self.last_emitted_attr
            || self.current_fg != self.last_emitted_fg
            || self.current_bg != self.last_emitted_bg;

        if !style_changed {
            return Ok(());
        }

        // Performance optimization: use SmallVec (stack-allocated)
        self.style_sequence_buf.clear();
        let mut needs_separator = false;

        // If any attribute changed, we need to reset and re-apply all
        // (ANSI doesn't support selective attribute removal)
        if self.current_attr != self.last_emitted_attr {
            // Reset all attributes first
            if self.last_emitted_attr != Attr::NORMAL {
                self.style_sequence_buf.push(b'0');
                needs_separator = true;
            }

            // Add current attribute codes
            if !self.current_attr.is_empty() {
                for code in self.current_attr.to_ansi_codes() {
                    if needs_separator {
                        self.style_sequence_buf.push(b';');
                    }
                    self.style_sequence_buf.extend_from_slice(code.as_bytes());
                    needs_separator = true;
                }
            }
        }

        // Add color codes if changed (using temporary buffer for String conversion)
        let mut color_buf = String::with_capacity(20);
        if self.current_fg != self.last_emitted_fg {
            if needs_separator {
                self.style_sequence_buf.push(b';');
            }
            color_buf.clear();
            self.current_fg.write_ansi_fg(&mut color_buf);
            self.style_sequence_buf
                .extend_from_slice(color_buf.as_bytes());
            needs_separator = true;
        }
        if self.current_bg != self.last_emitted_bg {
            if needs_separator {
                self.style_sequence_buf.push(b';');
            }
            color_buf.clear();
            self.current_bg.write_ansi_bg(&mut color_buf);
            self.style_sequence_buf
                .extend_from_slice(color_buf.as_bytes());
        }

        if !self.style_sequence_buf.is_empty() {
            self.buffer.push_str("\x1b[");
            self.buffer
                .push_str(std::str::from_utf8(&self.style_sequence_buf).unwrap());
            self.buffer.push('m');
        }

        // Update last emitted state
        self.last_emitted_attr = self.current_attr;
        self.last_emitted_fg = self.current_fg;
        self.last_emitted_bg = self.current_bg;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_window_creation() {
        let win = Window::new(10, 20, 5, 5).unwrap();
        assert_eq!(win.get_size(), (10, 20));
        assert_eq!(win.get_position(), (5, 5));
    }

    #[test]
    fn test_window_cursor_movement() {
        let mut win = Window::new(10, 20, 0, 0).unwrap();
        win.move_cursor(5, 10).unwrap();
        assert_eq!(win.cursor_y, 5);
        assert_eq!(win.cursor_x, 10);
    }

    #[test]
    fn test_window_invalid_cursor() {
        let mut win = Window::new(10, 20, 0, 0).unwrap();
        let result = win.move_cursor(15, 10);
        assert!(matches!(result, Err(Error::InvalidCoordinates { .. })));
    }

    #[test]
    fn test_window_print() {
        let mut win = Window::new(10, 20, 0, 0).unwrap();
        win.print("Hello").unwrap();
        assert_eq!(win.cursor_x, 5);
    }

    #[test]
    fn test_window_print_truncation() {
        let mut win = Window::new(10, 20, 0, 0).unwrap();
        win.move_cursor(0, 15).unwrap();
        // Only 5 chars can fit
        win.print("HelloWorld").unwrap();
        assert_eq!(win.cursor_x, 20);
    }

    #[test]
    fn test_window_attributes() {
        let mut win = Window::new(10, 20, 0, 0).unwrap();
        win.attron(Attr::BOLD).unwrap();
        assert!(win.current_attr.contains(Attr::BOLD));

        win.attroff(Attr::BOLD).unwrap();
        assert!(!win.current_attr.contains(Attr::BOLD));
    }

    #[test]
    fn test_window_colors() {
        let mut win = Window::new(10, 20, 0, 0).unwrap();
        win.set_fg(Color::Red).unwrap();
        win.set_bg(Color::Blue).unwrap();

        assert_eq!(win.current_fg, Color::Red);
        assert_eq!(win.current_bg, Color::Blue);
    }

    #[test]
    fn test_window_clear() {
        let mut win = Window::new(10, 20, 0, 0).unwrap();
        win.cursor_x = 5;
        win.cursor_y = 5;
        win.clear().unwrap();
        assert_eq!(win.cursor_x, 0);
        assert_eq!(win.cursor_y, 0);
    }

    #[test]
    fn test_window_border_buffer() {
        let mut win = Window::new(5, 10, 0, 0).unwrap();
        win.border('|', '|', '-', '-', '+', '+', '+', '+').unwrap();
        // Just ensure it doesn't panic and generates output
        assert!(!win.buffer.is_empty());
    }

    #[test]
    fn test_scrollok() {
        let mut win = Window::new(10, 20, 0, 0).unwrap();

        // Default is disabled
        assert!(!win.scroll_enabled);

        // Enable scrolling
        win.scrollok(true).unwrap();
        assert!(win.scroll_enabled);

        // Disable scrolling
        win.scrollok(false).unwrap();
        assert!(!win.scroll_enabled);
    }

    #[test]
    fn test_scroll_disabled() {
        let mut win = Window::new(10, 20, 0, 0).unwrap();

        // Scrolling is disabled by default
        assert!(!win.scroll_enabled);

        // Should not generate any output when disabled
        win.scroll(5).unwrap();
        assert!(win.buffer.is_empty());

        win.scroll(-3).unwrap();
        assert!(win.buffer.is_empty());
    }

    #[test]
    fn test_scroll_up() {
        let mut win = Window::new(10, 20, 5, 5).unwrap();

        // Enable scrolling
        win.scrollok(true).unwrap();

        // Scroll up (positive value)
        win.scroll(1).unwrap();

        // Should generate ANSI escape sequences for scrolling
        assert!(!win.buffer.is_empty());
        assert!(win.buffer.contains("\x1b[")); // Contains escape sequence
    }

    #[test]
    fn test_scroll_down() {
        let mut win = Window::new(10, 20, 5, 5).unwrap();

        // Enable scrolling
        win.scrollok(true).unwrap();

        // Scroll down (negative value)
        win.scroll(-2).unwrap();

        // Should generate ANSI escape sequences for scrolling
        assert!(!win.buffer.is_empty());
        assert!(win.buffer.contains("\x1b[")); // Contains escape sequence
    }

    #[test]
    fn test_scroll_zero() {
        let mut win = Window::new(10, 20, 0, 0).unwrap();

        // Enable scrolling
        win.scrollok(true).unwrap();

        // Scroll zero lines (no-op)
        win.scroll(0).unwrap();

        // Should not generate any output
        assert!(win.buffer.is_empty());
    }

    #[test]
    fn test_scroll_multiple_lines() {
        let mut win = Window::new(10, 20, 0, 0).unwrap();

        // Enable scrolling
        win.scrollok(true).unwrap();

        // Scroll multiple lines
        win.scroll(3).unwrap();

        let output = win.buffer.clone();
        assert!(!output.is_empty());

        // Clear and test negative
        win.buffer.clear();
        win.scroll(-4).unwrap();

        assert!(!win.buffer.is_empty());
    }

    #[test]
    fn test_window_style_caching_no_redundant_codes() {
        let mut win = Window::new(10, 20, 0, 0).unwrap();

        // First print should emit style codes
        win.print("Hello").unwrap();
        win.buffer.clear();

        // Second print with same style should NOT emit style codes again
        win.print("World").unwrap();
        let second_output = win.buffer.clone();

        // Second output should not contain any ANSI escape sequences
        assert!(!second_output.contains("\x1b["));
        assert_eq!(second_output, "World");
    }

    #[test]
    fn test_window_style_caching_emits_on_change() {
        let mut win = Window::new(10, 20, 0, 0).unwrap();

        // Print without style
        win.print("Normal").unwrap();
        win.buffer.clear();

        // Change to bold
        win.attron(Attr::BOLD).unwrap();
        win.print("Bold").unwrap();

        // Should contain bold code (1)
        assert!(win.buffer.contains("\x1b[1m"));
    }

    #[test]
    fn test_window_style_caching_color_change() {
        let mut win = Window::new(10, 20, 0, 0).unwrap();

        // Set foreground color
        win.set_fg(Color::Red).unwrap();
        win.print("Red").unwrap();
        win.buffer.clear();

        // Print with same color - no new codes
        win.print("AlsoRed").unwrap();
        assert!(!win.buffer.contains("\x1b["));

        // Change color
        win.buffer.clear();
        win.set_fg(Color::Blue).unwrap();
        win.print("Blue").unwrap();

        // Should contain new color code
        assert!(win.buffer.contains("\x1b["));
    }

    #[test]
    fn test_window_style_caching_attr_reset() {
        let mut win = Window::new(10, 20, 0, 0).unwrap();

        // Turn on bold
        win.attron(Attr::BOLD).unwrap();
        win.print("Bold").unwrap();
        win.buffer.clear();

        // Turn off bold (back to NORMAL)
        win.attroff(Attr::BOLD).unwrap();
        win.print("Normal").unwrap();

        // Should contain reset code (0)
        assert!(win.buffer.contains("\x1b[0m"));
    }

    #[test]
    fn test_window_style_caching_multiple_attrs() {
        let mut win = Window::new(10, 20, 0, 0).unwrap();

        // Turn on bold and underline
        win.attron(Attr::BOLD | Attr::UNDERLINE).unwrap();
        win.print("Styled").unwrap();
        win.buffer.clear();

        // Print again with same attrs - no codes
        win.print("AlsoStyled").unwrap();
        assert!(!win.buffer.contains("\x1b["));
        assert_eq!(win.buffer, "AlsoStyled");
    }

    #[test]
    fn test_window_buffer_preallocation() {
        // Create a window
        let win = Window::new(10, 20, 0, 0).unwrap();

        // Verify buffer has non-zero capacity
        assert!(win.buffer.capacity() > 0);
        // Should be at least 10 * 20 * 10 = 2000 bytes
        assert!(win.buffer.capacity() >= 2000);
    }

    #[test]
    fn test_window_buffer_capacity_capped() {
        // Create a very large window
        let win = Window::new(1000, 1000, 0, 0).unwrap();

        // Verify capacity is capped at 64KB even for large windows
        assert_eq!(win.buffer.capacity(), 65536);
    }

    #[test]
    fn test_window_buffer_no_reallocation_on_typical_use() {
        let mut win = Window::new(10, 20, 0, 0).unwrap();
        let initial_capacity = win.buffer.capacity();

        // Perform typical operations
        for i in 0..5 {
            win.mvprint(i, 0, "Test line").unwrap();
        }

        // Buffer should not have reallocated
        assert_eq!(win.buffer.capacity(), initial_capacity);
    }

    #[test]
    fn test_window_cursor_movement_short_horizontal() {
        let mut win = Window::new(10, 20, 5, 5).unwrap();
        win.cursor_x = 5;
        win.cursor_y = 3;

        // Move forward 2 cells (should use CUF)
        win.move_cursor(3, 7).unwrap();
        assert!(win.buffer.contains("\x1b[2C")); // Cursor Forward 2
        assert_eq!(win.cursor_x, 7);
        assert_eq!(win.cursor_y, 3);
    }

    #[test]
    fn test_window_cursor_movement_short_vertical() {
        let mut win = Window::new(10, 20, 5, 5).unwrap();
        win.cursor_x = 5;
        win.cursor_y = 3;

        // Move down 2 lines (should use CUD)
        win.move_cursor(5, 5).unwrap();
        assert!(win.buffer.contains("\x1b[2B")); // Cursor Down 2
        assert_eq!(win.cursor_x, 5);
        assert_eq!(win.cursor_y, 5);
    }

    #[test]
    fn test_window_cursor_movement_long_distance() {
        let mut win = Window::new(10, 20, 5, 5).unwrap();
        win.cursor_x = 2;
        win.cursor_y = 1;

        // Move 10 cells forward (should use CUP)
        win.move_cursor(1, 12).unwrap();
        // abs_y = 5 + 1 = 6, abs_x = 5 + 12 = 17
        // In 1-based: row 7, col 18
        assert!(win.buffer.contains("\x1b[7;18H")); // CUP
        assert_eq!(win.cursor_x, 12);
        assert_eq!(win.cursor_y, 1);
    }

    #[test]
    fn test_window_cursor_movement_diagonal() {
        let mut win = Window::new(10, 20, 0, 0).unwrap();
        win.cursor_x = 5;
        win.cursor_y = 3;

        // Diagonal movement (should use CUP)
        win.move_cursor(5, 8).unwrap();
        assert!(win.buffer.contains("\x1b[6;9H")); // CUP
        assert_eq!(win.cursor_x, 8);
        assert_eq!(win.cursor_y, 5);
    }

    #[test]
    fn test_window_rle_long_blank_run() {
        let mut win = Window::new(10, 20, 0, 0).unwrap();

        // Print 15 spaces (should use ECH)
        win.print("               ").unwrap();
        assert!(win.buffer.contains("\x1b[15X")); // ECH sequence
        assert_eq!(win.cursor_x, 15);
    }

    #[test]
    fn test_window_rle_short_blank_run() {
        let mut win = Window::new(10, 20, 0, 0).unwrap();

        // Print 5 spaces (should use regular output)
        win.print("     ").unwrap();
        assert!(!win.buffer.contains("\x1b[")); // Should NOT use ECH
        assert_eq!(win.buffer, "     ");
        assert_eq!(win.cursor_x, 5);
    }

    #[test]
    fn test_window_rle_threshold_8_spaces() {
        let mut win = Window::new(10, 20, 0, 0).unwrap();

        // Print exactly 8 spaces (should use ECH)
        win.print("        ").unwrap();
        assert!(win.buffer.contains("\x1b[8X"));
        assert_eq!(win.cursor_x, 8);
    }

    #[test]
    fn test_window_rle_with_truncation() {
        let mut win = Window::new(10, 20, 0, 0).unwrap();
        win.cursor_x = 15; // Near end of window

        // Print 10 spaces, but only 5 will fit
        win.print("          ").unwrap();
        // Should NOT use ECH because truncated length is only 5
        assert!(!win.buffer.contains("\x1b[")); // Should NOT use ECH
        assert_eq!(win.cursor_x, 20);
    }

    #[test]
    fn test_window_rle_non_blank_text() {
        let mut win = Window::new(10, 20, 0, 0).unwrap();

        // Print regular text
        win.print("Hello").unwrap();
        assert!(!win.buffer.contains("\x1b[")); // No escape sequences
        assert_eq!(win.buffer, "Hello");
        assert_eq!(win.cursor_x, 5);
    }
}
