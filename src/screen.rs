use crate::attr::Attr;
use crate::backend::Backend;
use crate::color::{Color, ColorPair};
use crate::error::{Error, Result};
use crate::input::Key;
use crate::window::Window;
use std::collections::HashMap;
use std::fmt::Write;
use std::io;

/// Main screen interface
pub struct Screen {
    cursor_x: u16,
    cursor_y: u16,
    current_attr: Attr,
    current_fg: Option<Color>,
    current_bg: Option<Color>,
    color_pairs: HashMap<u8, ColorPair>,
    cursor_visible: bool,
    buffer: String,
    // Performance optimization: track last emitted style to avoid redundant codes
    last_emitted_attr: Attr,
    last_emitted_fg: Option<Color>,
    last_emitted_bg: Option<Color>,
    // Performance optimization: reuse style code buffer to avoid allocations
    style_code_buffer: Vec<String>,
}

impl Screen {
    /// Initialize the screen
    pub fn init() -> Result<Self> {
        Backend::init()?;

        // Performance optimization: pre-allocate buffer based on terminal size
        // Estimate: ~10 bytes per cell (ANSI codes + character)
        let (rows, cols) = Backend::get_terminal_size().unwrap_or((24, 80));
        let estimated_capacity = (rows as usize * cols as usize * 10).min(65536); // Cap at 64KB

        Ok(Self {
            cursor_x: 0,
            cursor_y: 0,
            current_attr: Attr::NORMAL,
            current_fg: None,
            current_bg: None,
            color_pairs: HashMap::new(),
            cursor_visible: false,
            buffer: String::with_capacity(estimated_capacity),
            last_emitted_attr: Attr::NORMAL,
            last_emitted_fg: None,
            last_emitted_bg: None,
            style_code_buffer: Vec::with_capacity(5), // Pre-allocate for typical usage
        })
    }

    /// Clean up and restore terminal
    pub fn endwin(self) -> Result<()> {
        Backend::cleanup()
    }

    /// Get terminal size (rows, cols)
    pub fn get_size(&self) -> Result<(u16, u16)> {
        Backend::get_terminal_size()
    }

    /// Move cursor to position (y, x)
    pub fn move_cursor(&mut self, y: u16, x: u16) -> Result<()> {
        // Performance optimization: use relative cursor movement for short distances
        let dy = (y as i32 - self.cursor_y as i32).abs();
        let dx = (x as i32 - self.cursor_x as i32).abs();

        // Threshold: use relative movement if distance < 4 cells
        // (relative sequences are shorter for small distances)
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
            write!(self.buffer, "\x1b[{};{}H", y + 1, x + 1)?; // CUP - Cursor Position
        }

        self.cursor_y = y;
        self.cursor_x = x;
        Ok(())
    }

    /// Print text at current cursor position
    pub fn print(&mut self, text: &str) -> Result<()> {
        // Performance optimization: use ECH (Erase Character) for long blank runs
        if text.len() >= 8 && text.chars().all(|c| c == ' ') {
            // Use ECH sequence for efficiency
            write!(self.buffer, "\x1b[{}X", text.len())?;
            self.cursor_x += text.len() as u16;
            return Ok(());
        }

        self.apply_style()?;
        write!(self.buffer, "{}", text)?;
        self.cursor_x += text.len() as u16;
        Ok(())
    }

    /// Move cursor and print (like mvprintw)
    pub fn mvprint(&mut self, y: u16, x: u16, text: &str) -> Result<()> {
        self.move_cursor(y, x)?;
        self.print(text)
    }

    /// Add a single character
    pub fn addch(&mut self, ch: char) -> Result<()> {
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

    /// Set attributes
    pub fn attrset(&mut self, attr: Attr) -> Result<()> {
        self.current_attr = attr;
        Ok(())
    }

    /// Initialize a color pair
    pub fn init_pair(&mut self, pair: u8, fg: Color, bg: Color) -> Result<()> {
        self.color_pairs.insert(pair, ColorPair::new(fg, bg));
        Ok(())
    }

    /// Set current color pair
    pub fn color_pair(&mut self, pair: u8) -> Result<()> {
        let color_pair = self
            .color_pairs
            .get(&pair)
            .ok_or(Error::InvalidColorPair(pair))?;
        self.current_fg = Some(color_pair.fg);
        self.current_bg = Some(color_pair.bg);
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

    /// Clear the entire screen
    pub fn clear(&mut self) -> Result<()> {
        write!(self.buffer, "\x1b[2J")?;
        self.move_cursor(0, 0)?;
        Ok(())
    }

    /// Clear to end of line
    pub fn clrtoeol(&mut self) -> Result<()> {
        write!(self.buffer, "\x1b[K")?;
        Ok(())
    }

    /// Clear to bottom of screen
    pub fn clrtobot(&mut self) -> Result<()> {
        write!(self.buffer, "\x1b[J")?;
        Ok(())
    }

    /// Set cursor visibility
    pub fn cursor_visible(&mut self, visible: bool) -> Result<()> {
        self.cursor_visible = visible;
        if visible {
            write!(self.buffer, "\x1b[?25h")?;
        } else {
            write!(self.buffer, "\x1b[?25l")?;
        }
        Ok(())
    }

    /// Draw a box border
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
        let (rows, cols) = self.get_size()?;

        // Top border
        self.mvaddch(0, 0, tl)?;
        for _ in 1..cols - 1 {
            self.addch(ts)?;
        }
        self.addch(tr)?;

        // Sides
        for y in 1..rows - 1 {
            self.mvaddch(y, 0, ls)?;
            self.mvaddch(y, cols - 1, rs)?;
        }

        // Bottom border
        self.mvaddch(rows - 1, 0, bl)?;
        for _ in 1..cols - 1 {
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

    /// Read a single key
    pub fn getch(&mut self) -> Result<Key> {
        self.refresh()?;
        Backend::read_key()
    }

    /// Read a key with timeout (in milliseconds). Returns None if timeout expires.
    pub fn getch_timeout(&mut self, timeout_ms: u64) -> Result<Option<Key>> {
        self.refresh()?;
        Backend::read_key_timeout(Some(timeout_ms))
    }

    /// Refresh the screen (flush buffer to stdout)
    pub fn refresh(&mut self) -> Result<()> {
        use std::io::Write as IoWrite;
        io::stdout().write_all(self.buffer.as_bytes())?;
        io::stdout().flush()?;
        self.buffer.clear();
        Ok(())
    }

    /// Update internal buffer without refreshing screen
    pub fn wnoutrefresh(&mut self) -> Result<()> {
        Backend::add_to_update_buffer(&self.buffer)?;
        self.buffer.clear();
        Ok(())
    }

    /// Update physical screen with all pending changes
    pub fn doupdate() -> Result<()> {
        Backend::doupdate()
    }

    /// Enable Kitty keyboard protocol with the specified flags
    pub fn enable_kitty_keyboard(&mut self, flags: crate::kitty::KittyFlags) -> Result<()> {
        write!(self.buffer, "{}", crate::kitty::enable_sequence(flags))?;
        Ok(())
    }

    /// Disable Kitty keyboard protocol
    pub fn disable_kitty_keyboard(&mut self) -> Result<()> {
        write!(self.buffer, "{}", crate::kitty::disable_sequence())?;
        Ok(())
    }

    /// Push current keyboard mode and enable Kitty keyboard protocol
    pub fn push_kitty_keyboard(&mut self, flags: crate::kitty::KittyFlags) -> Result<()> {
        write!(self.buffer, "{}", crate::kitty::push_sequence(flags))?;
        Ok(())
    }

    /// Pop keyboard mode (restore previous mode)
    pub fn pop_kitty_keyboard(&mut self) -> Result<()> {
        write!(self.buffer, "{}", crate::kitty::pop_sequence())?;
        Ok(())
    }

    /// Display an image using Kitty graphics protocol
    pub fn display_kitty_image(&mut self, image: &crate::image::KittyImage) -> Result<()> {
        let seq = image.to_sequence().map_err(|_| {
            Error::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                "image encoding error",
            ))
        })?;
        write!(self.buffer, "{}", seq)?;
        Ok(())
    }

    /// Display an image using Sixel graphics protocol
    pub fn display_sixel_image(&mut self, image: &crate::image::SixelImage) -> Result<()> {
        let seq = image.to_sequence().map_err(|_| {
            Error::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                "image encoding error",
            ))
        })?;
        write!(self.buffer, "{}", seq)?;
        Ok(())
    }

    /// Delete a Kitty image by ID
    pub fn delete_kitty_image(&mut self, image_id: u32) -> Result<()> {
        write!(
            self.buffer,
            "{}",
            crate::image::delete_kitty_image(image_id)
        )?;
        Ok(())
    }

    /// Delete all Kitty images
    pub fn delete_all_kitty_images(&mut self) -> Result<()> {
        write!(self.buffer, "{}", crate::image::delete_all_kitty_images())?;
        Ok(())
    }

    /// Create a new window
    pub fn newwin(&self, height: u16, width: u16, y: u16, x: u16) -> Result<Window> {
        if height == 0 || width == 0 {
            return Err(Error::InvalidDimensions { height, width });
        }
        Window::new(height, width, y, x)
    }

    fn apply_style(&mut self) -> Result<()> {
        // Performance optimization: only emit ANSI codes if style changed since last emission
        let style_changed = self.current_attr != self.last_emitted_attr
            || self.current_fg != self.last_emitted_fg
            || self.current_bg != self.last_emitted_bg;

        if !style_changed {
            return Ok(());
        }

        // Performance optimization: reuse allocated buffer
        self.style_code_buffer.clear();

        // If any attribute changed, we need to reset and re-apply all
        // (ANSI doesn't support selective attribute removal)
        if self.current_attr != self.last_emitted_attr {
            // Reset all attributes first
            if self.last_emitted_attr != Attr::NORMAL {
                self.style_code_buffer.push("0".to_string());
            }

            // Add current attribute codes
            if !self.current_attr.is_empty() {
                self.style_code_buffer.extend(
                    self.current_attr
                        .to_ansi_codes()
                        .iter()
                        .map(|s| s.to_string()),
                );
            }
        }

        // Add color codes if changed
        if self.current_fg != self.last_emitted_fg {
            if let Some(fg) = &self.current_fg {
                self.style_code_buffer.push(fg.to_ansi_fg());
            }
        }
        if self.current_bg != self.last_emitted_bg {
            if let Some(bg) = &self.current_bg {
                self.style_code_buffer.push(bg.to_ansi_bg());
            }
        }

        if !self.style_code_buffer.is_empty() {
            write!(self.buffer, "\x1b[{}m", self.style_code_buffer.join(";")).map_err(|_| {
                Error::Io(std::io::Error::new(std::io::ErrorKind::Other, "fmt error"))
            })?;
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
    fn test_screen_buffer_operations() {
        // These tests don't actually initialize the terminal
        let mut scr = Screen {
            cursor_x: 0,
            cursor_y: 0,
            current_attr: Attr::NORMAL,
            current_fg: None,
            current_bg: None,
            color_pairs: HashMap::new(),
            cursor_visible: false,
            buffer: String::new(),
            last_emitted_attr: Attr::NORMAL,
            last_emitted_fg: None,
            last_emitted_bg: None,
            style_code_buffer: Vec::new(),
        };

        scr.move_cursor(5, 10).unwrap();
        assert!(scr.buffer.contains("\x1b[6;11H"));
        assert_eq!(scr.cursor_x, 10);
        assert_eq!(scr.cursor_y, 5);

        scr.buffer.clear();
        scr.cursor_x = 0; // Reset cursor for next test
        scr.print("Hello").unwrap();
        assert_eq!(scr.cursor_x, 5);
    }

    #[test]
    fn test_attributes() {
        let mut scr = Screen {
            cursor_x: 0,
            cursor_y: 0,
            current_attr: Attr::NORMAL,
            current_fg: None,
            current_bg: None,
            color_pairs: HashMap::new(),
            cursor_visible: false,
            buffer: String::new(),
            last_emitted_attr: Attr::NORMAL,
            last_emitted_fg: None,
            last_emitted_bg: None,
            style_code_buffer: Vec::new(),
        };

        scr.attron(Attr::BOLD).unwrap();
        assert!(scr.current_attr.contains(Attr::BOLD));

        scr.attron(Attr::UNDERLINE).unwrap();
        assert!(scr.current_attr.contains(Attr::BOLD | Attr::UNDERLINE));

        scr.attroff(Attr::BOLD).unwrap();
        assert!(!scr.current_attr.contains(Attr::BOLD));
        assert!(scr.current_attr.contains(Attr::UNDERLINE));
    }

    #[test]
    fn test_color_pairs() {
        let mut scr = Screen {
            cursor_x: 0,
            cursor_y: 0,
            current_attr: Attr::NORMAL,
            current_fg: None,
            current_bg: None,
            color_pairs: HashMap::new(),
            cursor_visible: false,
            buffer: String::new(),
            last_emitted_attr: Attr::NORMAL,
            last_emitted_fg: None,
            last_emitted_bg: None,
            style_code_buffer: Vec::new(),
        };

        scr.init_pair(1, Color::Red, Color::Black).unwrap();
        scr.color_pair(1).unwrap();

        assert_eq!(scr.current_fg, Some(Color::Red));
        assert_eq!(scr.current_bg, Some(Color::Black));
    }

    #[test]
    fn test_invalid_color_pair() {
        let mut scr = Screen {
            cursor_x: 0,
            cursor_y: 0,
            current_attr: Attr::NORMAL,
            current_fg: None,
            current_bg: None,
            color_pairs: HashMap::new(),
            cursor_visible: false,
            buffer: String::new(),
            last_emitted_attr: Attr::NORMAL,
            last_emitted_fg: None,
            last_emitted_bg: None,
            style_code_buffer: Vec::new(),
        };

        let result = scr.color_pair(99);
        assert!(matches!(result, Err(Error::InvalidColorPair(99))));
    }

    #[test]
    fn test_clear_operations() {
        let mut scr = Screen {
            cursor_x: 5,
            cursor_y: 5,
            current_attr: Attr::NORMAL,
            current_fg: None,
            current_bg: None,
            color_pairs: HashMap::new(),
            cursor_visible: false,
            buffer: String::new(),
            last_emitted_attr: Attr::NORMAL,
            last_emitted_fg: None,
            last_emitted_bg: None,
            style_code_buffer: Vec::new(),
        };

        scr.clear().unwrap();
        assert!(scr.buffer.contains("\x1b[2J"));
        assert_eq!(scr.cursor_x, 0);
        assert_eq!(scr.cursor_y, 0);

        scr.buffer.clear();
        scr.clrtoeol().unwrap();
        assert!(scr.buffer.contains("\x1b[K"));

        scr.buffer.clear();
        scr.clrtobot().unwrap();
        assert!(scr.buffer.contains("\x1b[J"));
    }

    #[test]
    fn test_cursor_visibility() {
        let mut scr = Screen {
            cursor_x: 0,
            cursor_y: 0,
            current_attr: Attr::NORMAL,
            current_fg: None,
            current_bg: None,
            color_pairs: HashMap::new(),
            cursor_visible: false,
            buffer: String::new(),
            last_emitted_attr: Attr::NORMAL,
            last_emitted_fg: None,
            last_emitted_bg: None,
            style_code_buffer: Vec::new(),
        };

        scr.cursor_visible(true).unwrap();
        assert!(scr.buffer.contains("\x1b[?25h"));

        scr.buffer.clear();
        scr.cursor_visible(false).unwrap();
        assert!(scr.buffer.contains("\x1b[?25l"));
    }

    #[test]
    fn test_enable_kitty_keyboard() {
        let mut scr = Screen {
            cursor_x: 0,
            cursor_y: 0,
            current_attr: Attr::NORMAL,
            current_fg: None,
            current_bg: None,
            color_pairs: HashMap::new(),
            cursor_visible: false,
            buffer: String::new(),
            last_emitted_attr: Attr::NORMAL,
            last_emitted_fg: None,
            last_emitted_bg: None,
            style_code_buffer: Vec::new(),
        };

        use crate::kitty::KittyFlags;

        // Test enable with default flags (DISAMBIGUATE)
        scr.enable_kitty_keyboard(KittyFlags::default()).unwrap();
        assert!(scr.buffer.contains("\x1b[>1u"));

        // Test enable with multiple flags
        scr.buffer.clear();
        scr.enable_kitty_keyboard(KittyFlags::DISAMBIGUATE | KittyFlags::EVENT_TYPES)
            .unwrap();
        assert!(scr.buffer.contains("\x1b[>3u"));
    }

    #[test]
    fn test_disable_kitty_keyboard() {
        let mut scr = Screen {
            cursor_x: 0,
            cursor_y: 0,
            current_attr: Attr::NORMAL,
            current_fg: None,
            current_bg: None,
            color_pairs: HashMap::new(),
            cursor_visible: false,
            buffer: String::new(),
            last_emitted_attr: Attr::NORMAL,
            last_emitted_fg: None,
            last_emitted_bg: None,
            style_code_buffer: Vec::new(),
        };

        scr.disable_kitty_keyboard().unwrap();
        assert_eq!(scr.buffer, "\x1b[<u");
    }

    #[test]
    fn test_push_pop_kitty_keyboard() {
        let mut scr = Screen {
            cursor_x: 0,
            cursor_y: 0,
            current_attr: Attr::NORMAL,
            current_fg: None,
            current_bg: None,
            color_pairs: HashMap::new(),
            cursor_visible: false,
            buffer: String::new(),
            last_emitted_attr: Attr::NORMAL,
            last_emitted_fg: None,
            last_emitted_bg: None,
            style_code_buffer: Vec::new(),
        };

        use crate::kitty::KittyFlags;

        // Test push
        scr.push_kitty_keyboard(KittyFlags::DISAMBIGUATE | KittyFlags::EVENT_TYPES)
            .unwrap();
        assert!(scr.buffer.contains("\x1b[>3;1u"));

        // Test pop
        scr.buffer.clear();
        scr.pop_kitty_keyboard().unwrap();
        assert_eq!(scr.buffer, "\x1b[<1u");
    }

    #[test]
    fn test_kitty_keyboard_flags_combination() {
        let mut scr = Screen {
            cursor_x: 0,
            cursor_y: 0,
            current_attr: Attr::NORMAL,
            current_fg: None,
            current_bg: None,
            color_pairs: HashMap::new(),
            cursor_visible: false,
            buffer: String::new(),
            last_emitted_attr: Attr::NORMAL,
            last_emitted_fg: None,
            last_emitted_bg: None,
            style_code_buffer: Vec::new(),
        };

        use crate::kitty::KittyFlags;

        // Test all flags enabled
        let all_flags = KittyFlags::DISAMBIGUATE
            | KittyFlags::EVENT_TYPES
            | KittyFlags::ALTERNATE_KEYS
            | KittyFlags::ALL_AS_ESCAPES
            | KittyFlags::REPORT_TEXT;

        scr.enable_kitty_keyboard(all_flags).unwrap();
        // 1+2+4+8+16 = 31
        assert!(scr.buffer.contains("\x1b[>31u"));
    }

    #[test]
    fn test_style_caching_no_redundant_codes() {
        let mut scr = Screen {
            cursor_x: 0,
            cursor_y: 0,
            current_attr: Attr::NORMAL,
            current_fg: None,
            current_bg: None,
            color_pairs: HashMap::new(),
            cursor_visible: false,
            buffer: String::new(),
            last_emitted_attr: Attr::NORMAL,
            last_emitted_fg: None,
            last_emitted_bg: None,
            style_code_buffer: Vec::new(),
        };

        // First print should emit style codes
        scr.print("Hello").unwrap();
        scr.buffer.clear();

        // Second print with same style should NOT emit style codes again
        scr.print("World").unwrap();
        let second_output = scr.buffer.clone();

        // Second output should not contain any ANSI escape sequences
        assert!(!second_output.contains("\x1b["));
        assert_eq!(second_output, "World");
    }

    #[test]
    fn test_style_caching_emits_on_change() {
        let mut scr = Screen {
            cursor_x: 0,
            cursor_y: 0,
            current_attr: Attr::NORMAL,
            current_fg: None,
            current_bg: None,
            color_pairs: HashMap::new(),
            cursor_visible: false,
            buffer: String::new(),
            last_emitted_attr: Attr::NORMAL,
            last_emitted_fg: None,
            last_emitted_bg: None,
            style_code_buffer: Vec::new(),
        };

        // Print without style
        scr.print("Normal").unwrap();
        scr.buffer.clear();

        // Change to bold
        scr.attron(Attr::BOLD).unwrap();
        scr.print("Bold").unwrap();

        // Should contain bold code (1)
        assert!(scr.buffer.contains("\x1b[1m"));
    }

    #[test]
    fn test_style_caching_color_change() {
        let mut scr = Screen {
            cursor_x: 0,
            cursor_y: 0,
            current_attr: Attr::NORMAL,
            current_fg: None,
            current_bg: None,
            color_pairs: HashMap::new(),
            cursor_visible: false,
            buffer: String::new(),
            last_emitted_attr: Attr::NORMAL,
            last_emitted_fg: None,
            last_emitted_bg: None,
            style_code_buffer: Vec::new(),
        };

        // Set foreground color
        scr.set_fg(Color::Red).unwrap();
        scr.print("Red").unwrap();
        scr.buffer.clear();

        // Print with same color - no new codes
        scr.print("AlsoRed").unwrap();
        assert!(!scr.buffer.contains("\x1b["));

        // Change color
        scr.buffer.clear();
        scr.set_fg(Color::Blue).unwrap();
        scr.print("Blue").unwrap();

        // Should contain new color code
        assert!(scr.buffer.contains("\x1b["));
    }

    #[test]
    fn test_style_caching_attr_reset() {
        let mut scr = Screen {
            cursor_x: 0,
            cursor_y: 0,
            current_attr: Attr::NORMAL,
            current_fg: None,
            current_bg: None,
            color_pairs: HashMap::new(),
            cursor_visible: false,
            buffer: String::new(),
            last_emitted_attr: Attr::NORMAL,
            last_emitted_fg: None,
            last_emitted_bg: None,
            style_code_buffer: Vec::new(),
        };

        // Turn on bold
        scr.attron(Attr::BOLD).unwrap();
        scr.print("Bold").unwrap();
        scr.buffer.clear();

        // Turn off bold (back to NORMAL)
        scr.attroff(Attr::BOLD).unwrap();
        scr.print("Normal").unwrap();

        // Should contain reset code (0)
        assert!(scr.buffer.contains("\x1b[0m"));
    }

    #[test]
    fn test_style_caching_multiple_attrs() {
        let mut scr = Screen {
            cursor_x: 0,
            cursor_y: 0,
            current_attr: Attr::NORMAL,
            current_fg: None,
            current_bg: None,
            color_pairs: HashMap::new(),
            cursor_visible: false,
            buffer: String::new(),
            last_emitted_attr: Attr::NORMAL,
            last_emitted_fg: None,
            last_emitted_bg: None,
            style_code_buffer: Vec::new(),
        };

        // Turn on bold and underline
        scr.attron(Attr::BOLD | Attr::UNDERLINE).unwrap();
        scr.print("Styled").unwrap();
        scr.buffer.clear();

        // Print again with same attrs - no codes
        scr.print("AlsoStyled").unwrap();
        assert!(!scr.buffer.contains("\x1b["));
        assert_eq!(scr.buffer, "AlsoStyled");
    }

    #[test]
    fn test_buffer_preallocation() {
        // Create a screen with pre-allocated buffer
        let scr = Screen {
            cursor_x: 0,
            cursor_y: 0,
            current_attr: Attr::NORMAL,
            current_fg: None,
            current_bg: None,
            color_pairs: HashMap::new(),
            cursor_visible: false,
            buffer: {
                let (rows, cols) = (24, 80);
                let estimated_capacity = (rows * cols * 10).min(65536);
                String::with_capacity(estimated_capacity)
            },
            last_emitted_attr: Attr::NORMAL,
            last_emitted_fg: None,
            last_emitted_bg: None,
            style_code_buffer: Vec::new(),
        };

        // Verify buffer has non-zero capacity
        assert!(scr.buffer.capacity() > 0);
        assert!(scr.buffer.capacity() >= 24 * 80 * 10);
    }

    #[test]
    fn test_buffer_capacity_capped() {
        // Test that very large terminal sizes don't result in excessive allocation
        let scr = Screen {
            cursor_x: 0,
            cursor_y: 0,
            current_attr: Attr::NORMAL,
            current_fg: None,
            current_bg: None,
            color_pairs: HashMap::new(),
            cursor_visible: false,
            buffer: {
                let (rows, cols) = (1000, 1000); // Very large terminal
                let estimated_capacity = (rows * cols * 10).min(65536);
                String::with_capacity(estimated_capacity)
            },
            last_emitted_attr: Attr::NORMAL,
            last_emitted_fg: None,
            last_emitted_bg: None,
            style_code_buffer: Vec::new(),
        };

        // Verify capacity is capped at 64KB
        assert_eq!(scr.buffer.capacity(), 65536);
    }

    #[test]
    fn test_buffer_no_reallocation_on_typical_use() {
        let mut scr = Screen {
            cursor_x: 0,
            cursor_y: 0,
            current_attr: Attr::NORMAL,
            current_fg: None,
            current_bg: None,
            color_pairs: HashMap::new(),
            cursor_visible: false,
            buffer: String::with_capacity(1000),
            last_emitted_attr: Attr::NORMAL,
            last_emitted_fg: None,
            last_emitted_bg: None,
            style_code_buffer: Vec::new(),
        };

        let initial_capacity = scr.buffer.capacity();

        // Perform typical operations
        for i in 0..10 {
            scr.move_cursor(i, 0).unwrap();
            scr.print("Test line").unwrap();
        }

        // Buffer should not have reallocated
        assert_eq!(scr.buffer.capacity(), initial_capacity);
    }

    #[test]
    fn test_cursor_movement_short_horizontal_forward() {
        let mut scr = Screen {
            cursor_x: 10,
            cursor_y: 5,
            current_attr: Attr::NORMAL,
            current_fg: None,
            current_bg: None,
            color_pairs: HashMap::new(),
            cursor_visible: false,
            buffer: String::new(),
            last_emitted_attr: Attr::NORMAL,
            last_emitted_fg: None,
            last_emitted_bg: None,
            style_code_buffer: Vec::new(),
        };

        // Move forward 2 cells (should use CUF)
        scr.move_cursor(5, 12).unwrap();
        assert!(scr.buffer.contains("\x1b[2C")); // Cursor Forward 2
        assert_eq!(scr.cursor_x, 12);
        assert_eq!(scr.cursor_y, 5);
    }

    #[test]
    fn test_cursor_movement_short_horizontal_back() {
        let mut scr = Screen {
            cursor_x: 10,
            cursor_y: 5,
            current_attr: Attr::NORMAL,
            current_fg: None,
            current_bg: None,
            color_pairs: HashMap::new(),
            cursor_visible: false,
            buffer: String::new(),
            last_emitted_attr: Attr::NORMAL,
            last_emitted_fg: None,
            last_emitted_bg: None,
            style_code_buffer: Vec::new(),
        };

        // Move back 3 cells (should use CUB)
        scr.move_cursor(5, 7).unwrap();
        assert!(scr.buffer.contains("\x1b[3D")); // Cursor Back 3
        assert_eq!(scr.cursor_x, 7);
        assert_eq!(scr.cursor_y, 5);
    }

    #[test]
    fn test_cursor_movement_short_vertical_down() {
        let mut scr = Screen {
            cursor_x: 10,
            cursor_y: 5,
            current_attr: Attr::NORMAL,
            current_fg: None,
            current_bg: None,
            color_pairs: HashMap::new(),
            cursor_visible: false,
            buffer: String::new(),
            last_emitted_attr: Attr::NORMAL,
            last_emitted_fg: None,
            last_emitted_bg: None,
            style_code_buffer: Vec::new(),
        };

        // Move down 2 lines (should use CUD)
        scr.move_cursor(7, 10).unwrap();
        assert!(scr.buffer.contains("\x1b[2B")); // Cursor Down 2
        assert_eq!(scr.cursor_x, 10);
        assert_eq!(scr.cursor_y, 7);
    }

    #[test]
    fn test_cursor_movement_short_vertical_up() {
        let mut scr = Screen {
            cursor_x: 10,
            cursor_y: 5,
            current_attr: Attr::NORMAL,
            current_fg: None,
            current_bg: None,
            color_pairs: HashMap::new(),
            cursor_visible: false,
            buffer: String::new(),
            last_emitted_attr: Attr::NORMAL,
            last_emitted_fg: None,
            last_emitted_bg: None,
            style_code_buffer: Vec::new(),
        };

        // Move up 1 line (should use CUU)
        scr.move_cursor(4, 10).unwrap();
        assert!(scr.buffer.contains("\x1b[1A")); // Cursor Up 1
        assert_eq!(scr.cursor_x, 10);
        assert_eq!(scr.cursor_y, 4);
    }

    #[test]
    fn test_cursor_movement_long_distance_uses_absolute() {
        let mut scr = Screen {
            cursor_x: 10,
            cursor_y: 5,
            current_attr: Attr::NORMAL,
            current_fg: None,
            current_bg: None,
            color_pairs: HashMap::new(),
            cursor_visible: false,
            buffer: String::new(),
            last_emitted_attr: Attr::NORMAL,
            last_emitted_fg: None,
            last_emitted_bg: None,
            style_code_buffer: Vec::new(),
        };

        // Move 10 cells forward (should use CUP for long distance)
        scr.move_cursor(5, 20).unwrap();
        assert!(scr.buffer.contains("\x1b[6;21H")); // CUP (note: +1 for 1-based indexing)
        assert_eq!(scr.cursor_x, 20);
        assert_eq!(scr.cursor_y, 5);
    }

    #[test]
    fn test_cursor_movement_diagonal_uses_absolute() {
        let mut scr = Screen {
            cursor_x: 10,
            cursor_y: 5,
            current_attr: Attr::NORMAL,
            current_fg: None,
            current_bg: None,
            color_pairs: HashMap::new(),
            cursor_visible: false,
            buffer: String::new(),
            last_emitted_attr: Attr::NORMAL,
            last_emitted_fg: None,
            last_emitted_bg: None,
            style_code_buffer: Vec::new(),
        };

        // Diagonal movement (should use CUP)
        scr.move_cursor(7, 12).unwrap();
        assert!(scr.buffer.contains("\x1b[8;13H")); // CUP
        assert_eq!(scr.cursor_x, 12);
        assert_eq!(scr.cursor_y, 7);
    }

    #[test]
    fn test_cursor_movement_same_position() {
        let mut scr = Screen {
            cursor_x: 10,
            cursor_y: 5,
            current_attr: Attr::NORMAL,
            current_fg: None,
            current_bg: None,
            color_pairs: HashMap::new(),
            cursor_visible: false,
            buffer: String::new(),
            last_emitted_attr: Attr::NORMAL,
            last_emitted_fg: None,
            last_emitted_bg: None,
            style_code_buffer: Vec::new(),
        };

        // Move to same position (should use CUP due to dx=0, dy=0)
        scr.move_cursor(5, 10).unwrap();
        assert!(scr.buffer.contains("\x1b[6;11H"));
        assert_eq!(scr.cursor_x, 10);
        assert_eq!(scr.cursor_y, 5);
    }

    #[test]
    fn test_rle_long_blank_run() {
        let mut scr = Screen {
            cursor_x: 0,
            cursor_y: 0,
            current_attr: Attr::NORMAL,
            current_fg: None,
            current_bg: None,
            color_pairs: HashMap::new(),
            cursor_visible: false,
            buffer: String::new(),
            last_emitted_attr: Attr::NORMAL,
            last_emitted_fg: None,
            last_emitted_bg: None,
            style_code_buffer: Vec::new(),
        };

        // Print 20 spaces (should use ECH)
        scr.print("                    ").unwrap();
        assert!(scr.buffer.contains("\x1b[20X")); // ECH sequence
        assert!(!scr.buffer.contains("    ")); // Should not contain actual spaces
        assert_eq!(scr.cursor_x, 20);
    }

    #[test]
    fn test_rle_short_blank_run() {
        let mut scr = Screen {
            cursor_x: 0,
            cursor_y: 0,
            current_attr: Attr::NORMAL,
            current_fg: None,
            current_bg: None,
            color_pairs: HashMap::new(),
            cursor_visible: false,
            buffer: String::new(),
            last_emitted_attr: Attr::NORMAL,
            last_emitted_fg: None,
            last_emitted_bg: None,
            style_code_buffer: Vec::new(),
        };

        // Print 5 spaces (should use regular output)
        scr.print("     ").unwrap();
        assert!(!scr.buffer.contains("\x1b[5X")); // Should NOT use ECH
        assert_eq!(scr.buffer, "     "); // Should contain actual spaces
        assert_eq!(scr.cursor_x, 5);
    }

    #[test]
    fn test_rle_non_blank_text() {
        let mut scr = Screen {
            cursor_x: 0,
            cursor_y: 0,
            current_attr: Attr::NORMAL,
            current_fg: None,
            current_bg: None,
            color_pairs: HashMap::new(),
            cursor_visible: false,
            buffer: String::new(),
            last_emitted_attr: Attr::NORMAL,
            last_emitted_fg: None,
            last_emitted_bg: None,
            style_code_buffer: Vec::new(),
        };

        // Print regular text (should use regular output)
        scr.print("Hello World").unwrap();
        assert!(!scr.buffer.contains("\x1b[")); // Should NOT use any escape sequences
        assert_eq!(scr.buffer, "Hello World");
        assert_eq!(scr.cursor_x, 11);
    }

    #[test]
    fn test_rle_threshold_exactly_8() {
        let mut scr = Screen {
            cursor_x: 0,
            cursor_y: 0,
            current_attr: Attr::NORMAL,
            current_fg: None,
            current_bg: None,
            color_pairs: HashMap::new(),
            cursor_visible: false,
            buffer: String::new(),
            last_emitted_attr: Attr::NORMAL,
            last_emitted_fg: None,
            last_emitted_bg: None,
            style_code_buffer: Vec::new(),
        };

        // Print exactly 8 spaces (should use ECH)
        scr.print("        ").unwrap();
        assert!(scr.buffer.contains("\x1b[8X")); // ECH sequence
        assert_eq!(scr.cursor_x, 8);
    }

    #[test]
    fn test_rle_threshold_7_spaces() {
        let mut scr = Screen {
            cursor_x: 0,
            cursor_y: 0,
            current_attr: Attr::NORMAL,
            current_fg: None,
            current_bg: None,
            color_pairs: HashMap::new(),
            cursor_visible: false,
            buffer: String::new(),
            last_emitted_attr: Attr::NORMAL,
            last_emitted_fg: None,
            last_emitted_bg: None,
            style_code_buffer: Vec::new(),
        };

        // Print exactly 7 spaces (should NOT use ECH)
        scr.print("       ").unwrap();
        assert!(!scr.buffer.contains("\x1b[")); // Should NOT use ECH
        assert_eq!(scr.buffer, "       ");
        assert_eq!(scr.cursor_x, 7);
    }

    #[test]
    fn test_style_code_buffer_reuse() {
        let mut scr = Screen {
            cursor_x: 0,
            cursor_y: 0,
            current_attr: Attr::NORMAL,
            current_fg: None,
            current_bg: None,
            color_pairs: HashMap::new(),
            cursor_visible: false,
            buffer: String::new(),
            last_emitted_attr: Attr::NORMAL,
            last_emitted_fg: None,
            last_emitted_bg: None,
            style_code_buffer: Vec::with_capacity(5),
        };

        // Apply first style
        scr.attron(Attr::BOLD).unwrap();
        scr.print("Bold").unwrap();

        // Buffer should have some capacity
        let capacity_after_first = scr.style_code_buffer.capacity();
        assert!(capacity_after_first >= 5);

        scr.buffer.clear();

        // Apply second style (should reuse buffer)
        scr.set_fg(Color::Red).unwrap();
        scr.print("Red").unwrap();

        // Capacity should not have changed (buffer reused)
        assert_eq!(scr.style_code_buffer.capacity(), capacity_after_first);

        scr.buffer.clear();

        // Apply third style (should still reuse buffer)
        scr.attroff(Attr::BOLD).unwrap();
        scr.print("Normal").unwrap();

        // Capacity should still not have changed
        assert_eq!(scr.style_code_buffer.capacity(), capacity_after_first);
    }

    #[test]
    fn test_style_code_buffer_cleared() {
        let mut scr = Screen {
            cursor_x: 0,
            cursor_y: 0,
            current_attr: Attr::NORMAL,
            current_fg: None,
            current_bg: None,
            color_pairs: HashMap::new(),
            cursor_visible: false,
            buffer: String::new(),
            last_emitted_attr: Attr::NORMAL,
            last_emitted_fg: None,
            last_emitted_bg: None,
            style_code_buffer: Vec::new(),
        };

        // Apply style
        scr.attron(Attr::BOLD).unwrap();
        scr.print("Bold").unwrap();

        // Buffer should be cleared after apply_style
        // (We can't directly check the private field, but we can verify behavior)
        scr.buffer.clear();

        // Apply same style again - should not emit codes
        scr.print("MoreBold").unwrap();
        assert!(!scr.buffer.contains("\x1b[")); // No escape codes
    }
}
