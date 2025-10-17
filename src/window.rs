use crate::attr::Attr;
use crate::color::Color;
use crate::error::{Error, Result};
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
    current_fg: Option<Color>,
    current_bg: Option<Color>,
    buffer: String,
    scroll_enabled: bool,
}

impl Window {
    pub(crate) fn new(height: u16, width: u16, y: u16, x: u16) -> Result<Self> {
        Ok(Self {
            height,
            width,
            begin_y: y,
            begin_x: x,
            cursor_x: 0,
            cursor_y: 0,
            current_attr: Attr::NORMAL,
            current_fg: None,
            current_bg: None,
            buffer: String::new(),
            scroll_enabled: false,
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

        self.cursor_y = y;
        self.cursor_x = x;

        let abs_y = self.begin_y + y;
        let abs_x = self.begin_x + x;
        write!(self.buffer, "\x1b[{};{}H", abs_y + 1, abs_x + 1)?;
        Ok(())
    }

    /// Print text at current cursor position
    pub fn print(&mut self, text: &str) -> Result<()> {
        self.apply_style()?;

        // Truncate text if it exceeds window width
        let remaining = (self.width - self.cursor_x) as usize;
        let text_to_print = if text.len() > remaining {
            &text[..remaining]
        } else {
            text
        };

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
        self.current_fg = Some(color);
        Ok(())
    }

    /// Set background color
    pub fn set_bg(&mut self, color: Color) -> Result<()> {
        self.current_bg = Some(color);
        Ok(())
    }

    /// Clear the window
    pub fn clear(&mut self) -> Result<()> {
        for y in 0..self.height {
            self.move_cursor(y, 0)?;
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
        let mut codes: Vec<String> = Vec::new();

        if !self.current_attr.is_empty() {
            codes.extend(
                self.current_attr
                    .to_ansi_codes()
                    .iter()
                    .map(|s| s.to_string()),
            );
        }

        if let Some(fg) = &self.current_fg {
            codes.push(fg.to_ansi_fg());
        }
        if let Some(bg) = &self.current_bg {
            codes.push(bg.to_ansi_bg());
        }

        if !codes.is_empty() {
            write!(self.buffer, "\x1b[{}m", codes.join(";")).map_err(|_| {
                Error::Io(std::io::Error::new(std::io::ErrorKind::Other, "fmt error"))
            })?;
        }

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

        assert_eq!(win.current_fg, Some(Color::Red));
        assert_eq!(win.current_bg, Some(Color::Blue));
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
}
