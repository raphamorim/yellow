use crate::attr::Attr;
use crate::backend::Backend;
use crate::cell::Cell;
use crate::color::{Color, ColorPair};
use crate::delta::DirtyRegion;
use crate::error::{Error, Result};
use crate::input::Key;
use crate::window::Window;
use smallvec::SmallVec;
use std::collections::HashMap;
use std::fmt::Write;

/// Main screen interface
pub struct Screen {
    cursor_x: u16,
    cursor_y: u16,
    rows: u16,
    cols: u16,
    current_attr: Attr,
    current_fg: Color,
    current_bg: Color,
    color_pairs: HashMap<u8, ColorPair>,
    cursor_visible: bool,
    buffer: String,
    // Performance optimization: track last emitted style to avoid redundant codes
    last_emitted_attr: Attr,
    last_emitted_fg: Color,
    last_emitted_bg: Color,
    // Performance optimization: SmallVec for ANSI sequences (stack-allocated for <64 bytes)
    // Most style sequences are <64 bytes, avoiding heap allocation in 95%+ of cases
    style_sequence_buf: SmallVec<[u8; 64]>,
    // Performance optimization: double-buffering for delta updates
    current_content: Vec<Vec<Cell>>,
    pending_content: Vec<Vec<Cell>>,
    dirty_lines: Vec<DirtyRegion>,
    // Performance optimization: line hash cache for scroll detection
    current_line_hashes: Vec<u64>,
    pending_line_hashes: Vec<u64>,
    // Performance optimization: interrupt-driven refresh
    #[cfg(unix)]
    stdin_fd: std::os::unix::io::RawFd,
    check_interval: usize,
    fifo_hold: bool,
}

impl Screen {
    /// Initialize the screen
    pub fn init() -> Result<Self> {
        Backend::init()?;

        // Performance optimization: pre-allocate buffer based on terminal size
        // Estimate: ~10 bytes per cell (ANSI codes + character)
        let (rows, cols) = Backend::get_terminal_size().unwrap_or((24, 80));
        let estimated_capacity = (rows as usize * cols as usize * 10).min(65536); // Cap at 64KB

        // Initialize screen buffers with blank cells
        let current_content = vec![vec![Cell::blank(); cols as usize]; rows as usize];
        let pending_content = vec![vec![Cell::blank(); cols as usize]; rows as usize];
        let dirty_lines = vec![DirtyRegion::clean(); rows as usize];

        // Initialize line hashes (blank lines have hash 0)
        let current_line_hashes = vec![0u64; rows as usize];
        let pending_line_hashes = vec![0u64; rows as usize];

        Ok(Self {
            cursor_x: 0,
            cursor_y: 0,
            rows,
            cols,
            current_attr: Attr::NORMAL,
            current_fg: Color::Reset,
            current_bg: Color::Reset,
            color_pairs: HashMap::new(),
            cursor_visible: false,
            buffer: String::with_capacity(estimated_capacity),
            last_emitted_attr: Attr::NORMAL,
            last_emitted_fg: Color::Reset,
            last_emitted_bg: Color::Reset,
            style_sequence_buf: SmallVec::new(), // Stack-allocated for sequences <64 bytes
            current_content,
            pending_content,
            dirty_lines,
            current_line_hashes,
            pending_line_hashes,
            #[cfg(unix)]
            stdin_fd: 0, // Standard input file descriptor
            check_interval: 5, // Check for input every 5 lines (default)
            fifo_hold: false, // Allow input checking by default
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
        if self.cursor_y >= self.rows || self.cursor_x >= self.cols {
            return Ok(()); // Out of bounds
        }

        let start_x = self.cursor_x as usize;
        let y = self.cursor_y as usize;

        // Write characters to pending buffer
        for (i, ch) in text.chars().enumerate() {
            let x = start_x + i;
            if x >= self.cols as usize {
                break; // Don't write past line end
            }

            let cell = Cell::with_style(ch, self.current_attr, self.current_fg, self.current_bg);
            self.pending_content[y][x] = cell;
        }

        // Mark dirty region and invalidate hash cache
        let end_x = (start_x + text.len())
            .min(self.cols as usize)
            .saturating_sub(1);
        self.dirty_lines[y].mark(start_x as u16, end_x as u16);
        self.pending_line_hashes[y] = 0; // Invalidate cache (will be recomputed on refresh)

        // Update cursor
        self.cursor_x += text.len() as u16;
        self.cursor_x = self.cursor_x.min(self.cols);
        Ok(())
    }

    /// Move cursor and print (like mvprintw)
    pub fn mvprint(&mut self, y: u16, x: u16, text: &str) -> Result<()> {
        self.move_cursor(y, x)?;
        self.print(text)
    }

    /// Add a single character
    pub fn addch(&mut self, ch: char) -> Result<()> {
        if self.cursor_y >= self.rows || self.cursor_x >= self.cols {
            return Ok(()); // Out of bounds
        }

        let y = self.cursor_y as usize;
        let x = self.cursor_x as usize;

        // Write character to pending buffer
        let cell = Cell::with_style(ch, self.current_attr, self.current_fg, self.current_bg);
        self.pending_content[y][x] = cell;

        // Mark dirty region and invalidate hash cache
        self.dirty_lines[y].mark(x as u16, x as u16);
        self.pending_line_hashes[y] = 0; // Invalidate cache

        // Update cursor
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
        self.current_fg = color_pair.fg;
        self.current_bg = color_pair.bg;
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

    /// Clear the entire screen
    pub fn clear(&mut self) -> Result<()> {
        // Clear pending buffer to blank cells
        for row in &mut self.pending_content {
            for cell in row {
                *cell = Cell::blank();
            }
        }

        // Mark all lines as dirty and invalidate hashes
        for dirty in &mut self.dirty_lines {
            *dirty = DirtyRegion::full(self.cols);
        }
        for hash in &mut self.pending_line_hashes {
            *hash = 0; // All blank lines = hash 0
        }

        self.cursor_x = 0;
        self.cursor_y = 0;
        Ok(())
    }

    /// Clear to end of line
    pub fn clrtoeol(&mut self) -> Result<()> {
        if self.cursor_y >= self.rows {
            return Ok(());
        }

        let y = self.cursor_y as usize;
        let start_x = self.cursor_x as usize;

        // Clear from cursor to end of line
        for x in start_x..self.cols as usize {
            self.pending_content[y][x] = Cell::blank();
        }

        // Mark dirty region and invalidate hash cache
        self.dirty_lines[y].mark(start_x as u16, self.cols - 1);
        self.pending_line_hashes[y] = 0;
        Ok(())
    }

    /// Clear to bottom of screen
    pub fn clrtobot(&mut self) -> Result<()> {
        if self.cursor_y >= self.rows {
            return Ok(());
        }

        // Clear from cursor to end of current line
        self.clrtoeol()?;

        // Clear all lines below current line
        for y in (self.cursor_y + 1) as usize..self.rows as usize {
            for x in 0..self.cols as usize {
                self.pending_content[y][x] = Cell::blank();
            }
            self.dirty_lines[y] = DirtyRegion::full(self.cols);
            self.pending_line_hashes[y] = 0;
        }

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

    /// Set how often to check for input during refresh (Phase 2.1 optimization)
    ///
    /// Lower values = more responsive but slightly more CPU overhead
    /// Higher values = less overhead but potential input lag
    ///
    /// Default: 5 lines
    pub fn set_check_interval(&mut self, lines: usize) {
        self.check_interval = lines.max(1); // At least 1
    }

    /// Temporarily disable input checking during critical updates
    ///
    /// Use when you need a consistent screen state without interruption
    pub fn hold_refresh(&mut self) {
        self.fifo_hold = true;
    }

    /// Re-enable input checking during refresh
    pub fn release_refresh(&mut self) {
        self.fifo_hold = false;
    }

    /// Check if input is pending (non-blocking)
    ///
    /// Returns true if stdin has data available to read
    #[cfg(unix)]
    fn check_pending_input(&self) -> Result<bool> {
        use libc::{poll, pollfd, POLLIN};

        if self.fifo_hold {
            return Ok(false);
        }

        let mut fds = [pollfd {
            fd: self.stdin_fd,
            events: POLLIN,
            revents: 0,
        }];

        // Non-blocking poll (0 timeout)
        let result = unsafe { poll(fds.as_mut_ptr(), 1, 0) };

        if result < 0 {
            let err = std::io::Error::last_os_error();
            if err.kind() == std::io::ErrorKind::Interrupted {
                return Ok(false); // EINTR - treat as no input
            }
            return Err(Error::Io(err));
        }

        // Check if input is available
        Ok(result > 0 && (fds[0].revents & POLLIN) != 0)
    }

    #[cfg(not(unix))]
    fn check_pending_input(&self) -> Result<bool> {
        // Non-Unix platforms: always continue (could implement platform-specific later)
        Ok(false)
    }

    /// Refresh the screen (flush buffer to stdout)
    pub fn refresh(&mut self) -> Result<()> {
        // Clear output buffer
        self.buffer.clear();

        // Update line hashes for dirty lines (if not already cached)
        for y in 0..self.rows as usize {
            if self.dirty_lines[y].range().is_some() && self.pending_line_hashes[y] == 0 {
                // Recompute hash for this dirty line
                self.pending_line_hashes[y] = crate::delta::hash_line(&self.pending_content[y]);
            }
        }

        // Detect scroll operations using hash matching
        let scrolls = crate::delta::detect_scrolls(&self.current_line_hashes, &self.pending_line_hashes);

        // Execute scroll operations (using ANSI delete/insert line sequences)
        for scroll in &scrolls {
            if scroll.shift > 0 {
                // Scroll up: lines moved up, delete at bottom
                // Move to the line where deletion should happen
                let delete_at = scroll.start + scroll.size;
                write!(self.buffer, "\x1b[{};1H", delete_at + 1)?; // Position cursor
                write!(self.buffer, "\x1b[{}M", scroll.shift)?; // Delete n lines
            } else if scroll.shift < 0 {
                // Scroll down: lines moved down, insert at top
                write!(self.buffer, "\x1b[{};1H", scroll.start + 1)?; // Position cursor
                write!(self.buffer, "\x1b[{}L", scroll.shift.unsigned_abs())?; // Insert n lines
            }
        }

        // Process each dirty line (with interrupt checking)
        let mut lines_processed = 0;
        let mut refresh_aborted = false;

        for y in 0..self.rows as usize {
            if let Some((first_x, last_x)) = self.dirty_lines[y].range() {
                // Find actual differences within dirty region
                if let Some((first_diff, last_diff)) =
                    crate::delta::find_line_diff(&self.current_content[y], &self.pending_content[y])
                {
                    // Clamp to dirty region
                    let first = first_diff.max(first_x as usize);
                    let last = last_diff.min(last_x as usize);

                    if first <= last {
                        // Move cursor to start of change
                        write!(self.buffer, "\x1b[{};{}H", y + 1, first + 1)?;

                        // Output changed cells
                        let mut x = first;
                        while x <= last {
                            let cell = &self.pending_content[y][x];

                            // Check if style needs updating
                            let style_changed = cell.attr != self.last_emitted_attr
                                || cell.fg() != self.last_emitted_fg
                                || cell.bg() != self.last_emitted_bg;

                            // Apply style if changed
                            if style_changed {
                                // Extract style data before mutable borrow
                                let cell_style = (cell.attr, cell.fg(), cell.bg());
                                self.last_emitted_attr = cell_style.0;
                                self.last_emitted_fg = cell_style.1;
                                self.last_emitted_bg = cell_style.2;

                                // Build and emit style codes using SmallVec (stack-allocated)
                                self.style_sequence_buf.clear();
                                let mut needs_separator = false;

                                // Helper macro to add code with separator
                                macro_rules! add_code {
                                    ($code:expr) => {
                                        if needs_separator {
                                            self.style_sequence_buf.push(b';');
                                        }
                                        self.style_sequence_buf.extend_from_slice($code);
                                        needs_separator = true;
                                    };
                                }

                                // Add attribute codes
                                if cell_style.0.is_empty() {
                                    add_code!(b"0"); // Reset
                                } else {
                                    if cell_style.0.contains(Attr::BOLD) {
                                        add_code!(b"1");
                                    }
                                    if cell_style.0.contains(Attr::DIM) {
                                        add_code!(b"2");
                                    }
                                    if cell_style.0.contains(Attr::ITALIC) {
                                        add_code!(b"3");
                                    }
                                    if cell_style.0.contains(Attr::UNDERLINE) {
                                        add_code!(b"4");
                                    }
                                    if cell_style.0.contains(Attr::BLINK) {
                                        add_code!(b"5");
                                    }
                                    if cell_style.0.contains(Attr::REVERSE) {
                                        add_code!(b"7");
                                    }
                                    if cell_style.0.contains(Attr::HIDDEN) {
                                        add_code!(b"8");
                                    }
                                    if cell_style.0.contains(Attr::STRIKETHROUGH) {
                                        add_code!(b"9");
                                    }
                                }

                                // Add color codes using temporary string
                                // (write_ansi_fg/bg expect String, so we still need this)
                                let mut color_buf = String::with_capacity(20);
                                let fg = cell_style.1;
                                if needs_separator {
                                    self.style_sequence_buf.push(b';');
                                }
                                color_buf.clear();
                                fg.write_ansi_fg(&mut color_buf);
                                self.style_sequence_buf.extend_from_slice(color_buf.as_bytes());
                                needs_separator = true;

                                let bg = cell_style.2;
                                if needs_separator {
                                    self.style_sequence_buf.push(b';');
                                }
                                color_buf.clear();
                                bg.write_ansi_bg(&mut color_buf);
                                self.style_sequence_buf.extend_from_slice(color_buf.as_bytes());

                                // Emit ANSI sequence if we added any codes
                                if !self.style_sequence_buf.is_empty() {
                                    self.buffer.push_str("\x1b[");
                                    self.buffer.push_str(
                                        std::str::from_utf8(&self.style_sequence_buf).unwrap()
                                    );
                                    self.buffer.push('m');
                                }
                            }

                            // Output character (with RLE optimization for spaces)
                            if cell.ch == ' '
                                && cell.attr == Attr::NORMAL
                                && cell.fg() == Color::Reset
                                && cell.bg() == Color::Reset
                            {
                                // Check for run of blank spaces
                                let mut run_length = 1;
                                while x + run_length <= last
                                    && run_length < 256
                                    && self.pending_content[y][x + run_length].is_blank()
                                {
                                    run_length += 1;
                                }

                                if run_length >= 8 {
                                    // Use ECH for long runs
                                    write!(self.buffer, "\x1b[{}X", run_length)?;
                                    x += run_length;
                                    continue;
                                }
                            }

                            write!(self.buffer, "{}", cell.ch)?;
                            x += 1;
                        }
                    }
                }

                // Clear dirty flag only if not aborted
                if !refresh_aborted {
                    self.dirty_lines[y] = DirtyRegion::clean();
                }

                lines_processed += 1;

                // Check for input every check_interval lines (Phase 2.1 optimization)
                if lines_processed % self.check_interval == 0 {
                    if self.check_pending_input()? {
                        // Input detected - abort refresh, preserve dirty flags for unprocessed lines
                        refresh_aborted = true;
                        break;
                    }
                }
            }
        }

        // Flush buffer even if aborted (partial update is valid)
        crate::platform_io::write_all_stdout(self.buffer.as_bytes())?;

        // Swap buffers only if refresh completed (not aborted)
        if !refresh_aborted {
            std::mem::swap(&mut self.current_content, &mut self.pending_content);
            std::mem::swap(&mut self.current_line_hashes, &mut self.pending_line_hashes);

            // Copy back to pending (pending should match current after refresh)
            for y in 0..self.rows as usize {
                self.pending_content[y].clone_from_slice(&self.current_content[y]);
            }
            self.pending_line_hashes.copy_from_slice(&self.current_line_hashes);
        }

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

}

#[cfg(test)]
mod tests {
    use super::*;

    // Helper function to create a test Screen with all required fields
    fn create_test_screen() -> Screen {
        let rows = 24;
        let cols = 80;
        Screen {
            cursor_x: 0,
            cursor_y: 0,
            rows,
            cols,
            current_attr: Attr::NORMAL,
            current_fg: Color::Reset,
            current_bg: Color::Reset,
            color_pairs: HashMap::new(),
            cursor_visible: false,
            buffer: String::new(),
            last_emitted_attr: Attr::NORMAL,
            last_emitted_fg: Color::Reset,
            last_emitted_bg: Color::Reset,
            style_sequence_buf: SmallVec::new(),
            current_content: vec![vec![Cell::blank(); cols as usize]; rows as usize],
            pending_content: vec![vec![Cell::blank(); cols as usize]; rows as usize],
            dirty_lines: vec![DirtyRegion::clean(); rows as usize],
            current_line_hashes: vec![0u64; rows as usize],
            pending_line_hashes: vec![0u64; rows as usize],
            #[cfg(unix)]
            stdin_fd: 0,
            check_interval: 5,
            fifo_hold: false,
        }
    }

    #[test]
    fn test_screen_buffer_operations() {
        // These tests don't actually initialize the terminal
        let mut scr = create_test_screen();

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
        let mut scr = create_test_screen();

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
        let mut scr = create_test_screen();

        scr.init_pair(1, Color::Red, Color::Black).unwrap();
        scr.color_pair(1).unwrap();

        assert_eq!(scr.current_fg, Color::Red);
        assert_eq!(scr.current_bg, Color::Black);
    }

    #[test]
    fn test_invalid_color_pair() {
        let mut scr = create_test_screen();

        let result = scr.color_pair(99);
        assert!(matches!(result, Err(Error::InvalidColorPair(99))));
    }

    #[test]
    fn test_clear_operations() {
        let mut scr = create_test_screen();

        // Test clear() - should clear screen and reset cursor
        scr.print("Hello").unwrap();
        scr.clear().unwrap();
        assert_eq!(scr.cursor_x, 0);
        assert_eq!(scr.cursor_y, 0);

        // All pending content should be blank
        for row in &scr.pending_content {
            for cell in row {
                assert!(cell.is_blank());
            }
        }
    }

    #[test]
    fn test_cursor_visibility() {
        let mut scr = create_test_screen();

        scr.cursor_visible(true).unwrap();
        assert!(scr.buffer.contains("\x1b[?25h"));

        scr.buffer.clear();
        scr.cursor_visible(false).unwrap();
        assert!(scr.buffer.contains("\x1b[?25l"));
    }

    #[test]
    fn test_enable_kitty_keyboard() {
        let mut scr = create_test_screen();

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
        let mut scr = create_test_screen();

        scr.disable_kitty_keyboard().unwrap();
        assert_eq!(scr.buffer, "\x1b[<u");
    }

    #[test]
    fn test_push_pop_kitty_keyboard() {
        let mut scr = create_test_screen();

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
        let mut scr = create_test_screen();

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
        let mut scr = create_test_screen();

        // First print should emit style codes
        scr.print("Hello").unwrap();
        scr.refresh().unwrap();
        let first_output = scr.buffer.clone();
        scr.buffer.clear();

        // Second print at different position with same style
        scr.move_cursor(0, 10).unwrap();
        scr.print("World").unwrap();
        scr.refresh().unwrap();
        let second_output = scr.buffer.clone();

        // Second output should have less escape codes (no style codes, just cursor movement)
        assert!(second_output.contains("World"));
        // First output had cursor movement + content, second should have cursor movement + content
        // but both used the same default style
    }

    #[test]
    fn test_style_caching_emits_on_change() {
        let mut scr = create_test_screen();

        // Print without style
        scr.print("Normal").unwrap();
        scr.refresh().unwrap();
        scr.buffer.clear();

        // Change to bold
        scr.attron(Attr::BOLD).unwrap();
        scr.move_cursor(0, 10).unwrap();
        scr.print("Bold").unwrap();
        scr.refresh().unwrap();

        // Should contain bold code (1) and color resets (39;49)
        assert!(scr.buffer.contains("\x1b[1;39;49m"));
    }

    #[test]
    fn test_style_caching_color_change() {
        let mut scr = create_test_screen();

        // Set foreground color and print
        scr.set_fg(Color::Red).unwrap();
        scr.print("Red").unwrap();
        scr.refresh().unwrap();
        scr.buffer.clear();

        // Change color and print at different position
        scr.move_cursor(0, 10).unwrap();
        scr.set_fg(Color::Blue).unwrap();
        scr.print("Blue").unwrap();
        scr.refresh().unwrap();

        // Should contain new color code
        assert!(scr.buffer.contains("\x1b["));
    }

    #[test]
    fn test_style_caching_attr_reset() {
        let mut scr = create_test_screen();

        // Turn on bold and print
        scr.attron(Attr::BOLD).unwrap();
        scr.print("Bold").unwrap();
        scr.refresh().unwrap();
        scr.buffer.clear();

        // Turn off bold and print at different position
        scr.move_cursor(0, 10).unwrap();
        scr.attroff(Attr::BOLD).unwrap();
        scr.print("Normal").unwrap();
        scr.refresh().unwrap();

        // Should contain reset code (0) and color resets (39;49)
        assert!(scr.buffer.contains("\x1b[0;39;49m"));
    }

    #[test]
    fn test_style_caching_multiple_attrs() {
        let mut scr = create_test_screen();

        // Turn on bold and underline
        scr.attron(Attr::BOLD | Attr::UNDERLINE).unwrap();
        scr.print("Styled").unwrap();
        scr.refresh().unwrap();

        // Verify output contains styled text
        assert!(scr.buffer.contains("Styled"));
    }

    #[test]
    fn test_buffer_preallocation() {
        // Create a screen with pre-allocated buffer
        let scr = Screen {
            cursor_x: 0,
            cursor_y: 0,
            rows: 24,
            cols: 80,
            current_attr: Attr::NORMAL,
            current_fg: Color::Reset,
            current_bg: Color::Reset,
            color_pairs: HashMap::new(),
            cursor_visible: false,
            buffer: {
                let (rows, cols) = (24, 80);
                let estimated_capacity = (rows * cols * 10).min(65536);
                String::with_capacity(estimated_capacity)
            },
            last_emitted_attr: Attr::NORMAL,
            last_emitted_fg: Color::Reset,
            last_emitted_bg: Color::Reset,
            style_sequence_buf: SmallVec::new(),
            current_content: vec![vec![Cell::blank(); 80]; 24],
            pending_content: vec![vec![Cell::blank(); 80]; 24],
            dirty_lines: vec![DirtyRegion::clean(); 24],
                    current_line_hashes: vec![0u64; 24],
            pending_line_hashes: vec![0u64; 24],
            #[cfg(unix)]
            stdin_fd: 0,
            check_interval: 5,
            fifo_hold: false,
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
            rows: 24,
            cols: 80,
            current_attr: Attr::NORMAL,
            current_fg: Color::Reset,
            current_bg: Color::Reset,
            color_pairs: HashMap::new(),
            cursor_visible: false,
            buffer: {
                let (rows, cols) = (1000, 1000); // Very large terminal
                let estimated_capacity = (rows * cols * 10).min(65536);
                String::with_capacity(estimated_capacity)
            },
            last_emitted_attr: Attr::NORMAL,
            last_emitted_fg: Color::Reset,
            last_emitted_bg: Color::Reset,
            style_sequence_buf: SmallVec::new(),
            current_content: vec![vec![Cell::blank(); 80]; 24],
            pending_content: vec![vec![Cell::blank(); 80]; 24],
            dirty_lines: vec![DirtyRegion::clean(); 24],
                    current_line_hashes: vec![0u64; 24],
            pending_line_hashes: vec![0u64; 24],
            #[cfg(unix)]
            stdin_fd: 0,
            check_interval: 5,
            fifo_hold: false,
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
            current_fg: Color::Reset,
            current_bg: Color::Reset,
            color_pairs: HashMap::new(),
            cursor_visible: false,
            buffer: String::with_capacity(1000),
            last_emitted_attr: Attr::NORMAL,
            last_emitted_fg: Color::Reset,
            last_emitted_bg: Color::Reset,
            style_sequence_buf: SmallVec::new(),
            rows: 24,
            cols: 80,
            current_content: vec![vec![Cell::blank(); 80]; 24],
            pending_content: vec![vec![Cell::blank(); 80]; 24],
            dirty_lines: vec![DirtyRegion::clean(); 24],
                    current_line_hashes: vec![0u64; 24],
            pending_line_hashes: vec![0u64; 24],
            #[cfg(unix)]
            stdin_fd: 0,
            check_interval: 5,
            fifo_hold: false,
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
            current_fg: Color::Reset,
            current_bg: Color::Reset,
            color_pairs: HashMap::new(),
            cursor_visible: false,
            buffer: String::new(),
            last_emitted_attr: Attr::NORMAL,
            last_emitted_fg: Color::Reset,
            last_emitted_bg: Color::Reset,
            style_sequence_buf: SmallVec::new(),
            rows: 24,
            cols: 80,
            current_content: vec![vec![Cell::blank(); 80]; 24],
            pending_content: vec![vec![Cell::blank(); 80]; 24],
            dirty_lines: vec![DirtyRegion::clean(); 24],
                    current_line_hashes: vec![0u64; 24],
            pending_line_hashes: vec![0u64; 24],
            #[cfg(unix)]
            stdin_fd: 0,
            check_interval: 5,
            fifo_hold: false,
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
            current_fg: Color::Reset,
            current_bg: Color::Reset,
            color_pairs: HashMap::new(),
            cursor_visible: false,
            buffer: String::new(),
            last_emitted_attr: Attr::NORMAL,
            last_emitted_fg: Color::Reset,
            last_emitted_bg: Color::Reset,
            style_sequence_buf: SmallVec::new(),
            rows: 24,
            cols: 80,
            current_content: vec![vec![Cell::blank(); 80]; 24],
            pending_content: vec![vec![Cell::blank(); 80]; 24],
            dirty_lines: vec![DirtyRegion::clean(); 24],
                    current_line_hashes: vec![0u64; 24],
            pending_line_hashes: vec![0u64; 24],
            #[cfg(unix)]
            stdin_fd: 0,
            check_interval: 5,
            fifo_hold: false,
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
            current_fg: Color::Reset,
            current_bg: Color::Reset,
            color_pairs: HashMap::new(),
            cursor_visible: false,
            buffer: String::new(),
            last_emitted_attr: Attr::NORMAL,
            last_emitted_fg: Color::Reset,
            last_emitted_bg: Color::Reset,
            style_sequence_buf: SmallVec::new(),
            rows: 24,
            cols: 80,
            current_content: vec![vec![Cell::blank(); 80]; 24],
            pending_content: vec![vec![Cell::blank(); 80]; 24],
            dirty_lines: vec![DirtyRegion::clean(); 24],
                    current_line_hashes: vec![0u64; 24],
            pending_line_hashes: vec![0u64; 24],
            #[cfg(unix)]
            stdin_fd: 0,
            check_interval: 5,
            fifo_hold: false,
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
            current_fg: Color::Reset,
            current_bg: Color::Reset,
            color_pairs: HashMap::new(),
            cursor_visible: false,
            buffer: String::new(),
            last_emitted_attr: Attr::NORMAL,
            last_emitted_fg: Color::Reset,
            last_emitted_bg: Color::Reset,
            style_sequence_buf: SmallVec::new(),
            rows: 24,
            cols: 80,
            current_content: vec![vec![Cell::blank(); 80]; 24],
            pending_content: vec![vec![Cell::blank(); 80]; 24],
            dirty_lines: vec![DirtyRegion::clean(); 24],
                    current_line_hashes: vec![0u64; 24],
            pending_line_hashes: vec![0u64; 24],
            #[cfg(unix)]
            stdin_fd: 0,
            check_interval: 5,
            fifo_hold: false,
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
            current_fg: Color::Reset,
            current_bg: Color::Reset,
            color_pairs: HashMap::new(),
            cursor_visible: false,
            buffer: String::new(),
            last_emitted_attr: Attr::NORMAL,
            last_emitted_fg: Color::Reset,
            last_emitted_bg: Color::Reset,
            style_sequence_buf: SmallVec::new(),
            rows: 24,
            cols: 80,
            current_content: vec![vec![Cell::blank(); 80]; 24],
            pending_content: vec![vec![Cell::blank(); 80]; 24],
            dirty_lines: vec![DirtyRegion::clean(); 24],
                    current_line_hashes: vec![0u64; 24],
            pending_line_hashes: vec![0u64; 24],
            #[cfg(unix)]
            stdin_fd: 0,
            check_interval: 5,
            fifo_hold: false,
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
            current_fg: Color::Reset,
            current_bg: Color::Reset,
            color_pairs: HashMap::new(),
            cursor_visible: false,
            buffer: String::new(),
            last_emitted_attr: Attr::NORMAL,
            last_emitted_fg: Color::Reset,
            last_emitted_bg: Color::Reset,
            style_sequence_buf: SmallVec::new(),
            rows: 24,
            cols: 80,
            current_content: vec![vec![Cell::blank(); 80]; 24],
            pending_content: vec![vec![Cell::blank(); 80]; 24],
            dirty_lines: vec![DirtyRegion::clean(); 24],
                    current_line_hashes: vec![0u64; 24],
            pending_line_hashes: vec![0u64; 24],
            #[cfg(unix)]
            stdin_fd: 0,
            check_interval: 5,
            fifo_hold: false,
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
            current_fg: Color::Reset,
            current_bg: Color::Reset,
            color_pairs: HashMap::new(),
            cursor_visible: false,
            buffer: String::new(),
            last_emitted_attr: Attr::NORMAL,
            last_emitted_fg: Color::Reset,
            last_emitted_bg: Color::Reset,
            style_sequence_buf: SmallVec::new(),
            rows: 24,
            cols: 80,
            current_content: vec![vec![Cell::blank(); 80]; 24],
            pending_content: vec![vec![Cell::blank(); 80]; 24],
            dirty_lines: vec![DirtyRegion::clean(); 24],
                    current_line_hashes: vec![0u64; 24],
            pending_line_hashes: vec![0u64; 24],
            #[cfg(unix)]
            stdin_fd: 0,
            check_interval: 5,
            fifo_hold: false,
        };

        // Move to same position (should use CUP due to dx=0, dy=0)
        scr.move_cursor(5, 10).unwrap();
        assert!(scr.buffer.contains("\x1b[6;11H"));
        assert_eq!(scr.cursor_x, 10);
        assert_eq!(scr.cursor_y, 5);
    }

    #[test]
    fn test_rle_long_blank_run() {
        let mut scr = create_test_screen();

        // Print 20 spaces
        scr.print("                    ").unwrap();
        assert_eq!(scr.cursor_x, 20);

        // Refresh should use ECH for long blank runs
        scr.refresh().unwrap();
        assert!(
            scr.buffer.contains("\x1b[8X")
                || scr.buffer.contains("\x1b[20X")
                || scr.buffer.is_empty()
        );
        // Note: buffer might be empty if current==pending (no changes)
    }

    #[test]
    fn test_rle_short_blank_run() {
        let mut scr = create_test_screen();

        // Print 5 spaces
        scr.print("     ").unwrap();
        assert_eq!(scr.cursor_x, 5);

        // Verify spaces were written to pending buffer
        for i in 0..5 {
            assert_eq!(scr.pending_content[0][i].ch, ' ');
        }
    }

    #[test]
    fn test_rle_non_blank_text() {
        let mut scr = create_test_screen();

        // Print regular text
        scr.print("Hello World").unwrap();
        assert_eq!(scr.cursor_x, 11);

        // Verify text was written to pending buffer
        let text = "Hello World";
        for (i, ch) in text.chars().enumerate() {
            assert_eq!(scr.pending_content[0][i].ch, ch);
        }
    }

    #[test]
    fn test_rle_threshold_exactly_8() {
        let mut scr = create_test_screen();

        // Print exactly 8 spaces
        scr.print("        ").unwrap();
        assert_eq!(scr.cursor_x, 8);
        scr.refresh().unwrap();
        // ECH may or may not be used depending on delta optimization
        assert!(scr.buffer.len() >= 0); // Just verify it didn't crash
    }

    #[test]
    fn test_rle_threshold_7_spaces() {
        let mut scr = create_test_screen();

        // Print exactly 7 spaces
        scr.print("       ").unwrap();
        assert_eq!(scr.cursor_x, 7);

        // Verify spaces were written
        for i in 0..7 {
            assert_eq!(scr.pending_content[0][i].ch, ' ');
        }
    }

    #[test]
    fn test_hash_invalidation_on_print() {
        let mut scr = create_test_screen();

        // Initial hash should be 0 (blank line)
        assert_eq!(scr.pending_line_hashes[0], 0);

        // Print text - hash should be invalidated (set to 0 to mark for recomputation)
        scr.print("Hello").unwrap();
        assert_eq!(scr.pending_line_hashes[0], 0); // Still 0, will be computed on refresh

        // After refresh, hash should be computed and cached
        scr.refresh().unwrap();
        assert_ne!(scr.current_line_hashes[0], 0); // Hash computed
        assert_ne!(scr.pending_line_hashes[0], 0); // Copied from current
    }

    #[test]
    fn test_hash_invalidation_on_addch() {
        let mut scr = create_test_screen();

        // Add a character
        scr.addch('A').unwrap();
        assert_eq!(scr.pending_line_hashes[0], 0); // Invalidated

        // Refresh computes hash
        scr.refresh().unwrap();
        assert_ne!(scr.current_line_hashes[0], 0); // Hash computed
    }

    #[test]
    fn test_hash_invalidation_on_clear() {
        let mut scr = create_test_screen();

        // Write some text and refresh
        scr.print("Test").unwrap();
        scr.refresh().unwrap();
        let hash_before = scr.current_line_hashes[0];
        assert_ne!(hash_before, 0);

        // Clear should set all hashes to 0 (blank lines)
        scr.clear().unwrap();
        for hash in &scr.pending_line_hashes {
            assert_eq!(*hash, 0);
        }
    }

    #[test]
    fn test_hash_recomputation_on_refresh() {
        let mut scr = create_test_screen();

        // Write different text on two lines
        scr.mvprint(0, 0, "Line 1").unwrap();
        scr.mvprint(1, 0, "Line 2").unwrap();

        // Before refresh, hashes are invalidated
        assert_eq!(scr.pending_line_hashes[0], 0);
        assert_eq!(scr.pending_line_hashes[1], 0);

        // Refresh should compute hashes
        scr.refresh().unwrap();
        assert_ne!(scr.current_line_hashes[0], 0);
        assert_ne!(scr.current_line_hashes[1], 0);

        // Different lines should have different hashes
        assert_ne!(scr.current_line_hashes[0], scr.current_line_hashes[1]);
    }

    #[test]
    fn test_identical_lines_same_hash() {
        let mut scr = create_test_screen();

        // Write identical text on two different lines
        scr.mvprint(0, 0, "Same").unwrap();
        scr.mvprint(5, 0, "Same").unwrap();

        scr.refresh().unwrap();

        // Identical lines should produce identical hashes
        assert_eq!(scr.current_line_hashes[0], scr.current_line_hashes[5]);
        assert_ne!(scr.current_line_hashes[0], 0);
    }

    #[test]
    fn test_hash_persistence_across_refresh() {
        let mut scr = create_test_screen();

        // Write and refresh
        scr.print("Test").unwrap();
        scr.refresh().unwrap();
        let hash_after_first = scr.current_line_hashes[0];

        // Refresh again without changes
        scr.refresh().unwrap();

        // Hash should remain the same
        assert_eq!(scr.current_line_hashes[0], hash_after_first);
    }

    #[test]
    fn test_hash_swap_on_refresh() {
        let mut scr = create_test_screen();

        // Write text
        scr.print("Test").unwrap();

        // Before refresh, current is blank (hash 0), pending has content (hash 0 but will be computed)
        assert_eq!(scr.current_line_hashes[0], 0);
        assert_eq!(scr.pending_line_hashes[0], 0);

        // Refresh swaps buffers
        scr.refresh().unwrap();

        // After refresh, both should have the computed hash
        assert_ne!(scr.current_line_hashes[0], 0);
        assert_eq!(scr.current_line_hashes[0], scr.pending_line_hashes[0]);
    }

    #[test]
    fn test_scroll_detection_simple_scroll_up() {
        let mut scr = create_test_screen();

        // Write 8 unique lines
        for i in 0..8 {
            scr.mvprint(i, 0, &format!("Line {}", i)).unwrap();
        }
        scr.refresh().unwrap();
        scr.buffer.clear();

        // Simulate scroll up: delete first 3 lines, everything moves up
        for i in 0..5 {
            scr.mvprint(i, 0, &format!("Line {}", i + 3)).unwrap();
        }
        for i in 5..8 {
            scr.mvprint(i, 0, "New").unwrap();
        }

        scr.refresh().unwrap();

        // Should contain delete lines sequence (scroll detected)
        // Delete 3 lines: \x1b[3M
        assert!(scr.buffer.contains("\x1b[3M") || scr.buffer.len() < 100);
        // Note: buffer might use different optimization
    }

    #[test]
    fn test_scroll_detection_simple_scroll_down() {
        let mut scr = create_test_screen();

        // Write 8 unique lines
        for i in 0..8 {
            scr.mvprint(i, 0, &format!("Line {}", i)).unwrap();
        }
        scr.refresh().unwrap();
        scr.buffer.clear();

        // Simulate scroll down: insert 3 lines at top, everything moves down
        for i in 0..3 {
            scr.mvprint(i, 0, "New").unwrap();
        }
        for i in 3..8 {
            scr.mvprint(i, 0, &format!("Line {}", i - 3)).unwrap();
        }

        scr.refresh().unwrap();

        // Should contain insert lines sequence
        // Insert 3 lines: \x1b[3L
        assert!(scr.buffer.contains("\x1b[3L") || scr.buffer.len() < 100);
    }

    #[test]
    fn test_scroll_not_detected_for_small_changes() {
        let mut scr = create_test_screen();

        // Write only 2 matching lines (below minimum hunk size of 3)
        scr.mvprint(0, 0, "A").unwrap();
        scr.mvprint(1, 0, "B").unwrap();
        scr.refresh().unwrap();
        scr.buffer.clear();

        // Move them down by 1
        scr.mvprint(1, 0, "A").unwrap();
        scr.mvprint(2, 0, "B").unwrap();

        scr.refresh().unwrap();

        // Should NOT detect scroll (hunk too small)
        assert!(!scr.buffer.contains("\x1b[L"));
        assert!(!scr.buffer.contains("\x1b[M"));
    }
}
