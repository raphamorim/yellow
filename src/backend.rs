use crate::error::{Error, Result};
use crate::input::Key;
use std::io::{self, Read, Write};
use std::sync::{Mutex, OnceLock};

static BACKEND: OnceLock<Mutex<Backend>> = OnceLock::new();
static UPDATE_BUFFER: OnceLock<Mutex<String>> = OnceLock::new();

pub(crate) struct Backend {
    original_termios: Option<Termios>,
    initialized: bool,
}

#[cfg(unix)]
use std::os::unix::io::AsRawFd;

#[cfg(unix)]
#[derive(Clone)]
struct Termios {
    termios: libc::termios,
}

#[cfg(not(unix))]
#[derive(Clone)]
struct Termios;

impl Backend {
    fn new() -> Self {
        Self {
            original_termios: None,
            initialized: false,
        }
    }

    pub(crate) fn init() -> Result<()> {
        let backend = BACKEND.get_or_init(|| Mutex::new(Backend::new()));
        let mut guard = backend.lock().unwrap();

        if guard.initialized {
            return Err(Error::AlreadyInitialized);
        }

        guard.enable_raw_mode()?;
        guard.initialized = true;

        // Enter alternate screen
        print!("\x1b[?1049h");
        // Hide cursor
        print!("\x1b[?25l");
        // Clear screen
        print!("\x1b[2J");
        io::stdout().flush()?;

        Ok(())
    }

    pub(crate) fn cleanup() -> Result<()> {
        let backend = BACKEND.get().ok_or(Error::NotInitialized)?;
        let mut guard = backend.lock().unwrap();

        if !guard.initialized {
            return Ok(());
        }

        // Show cursor
        print!("\x1b[?25h");
        // Exit alternate screen
        print!("\x1b[?1049l");
        io::stdout().flush()?;

        guard.disable_raw_mode()?;
        guard.initialized = false;

        Ok(())
    }

    #[cfg(unix)]
    fn enable_raw_mode(&mut self) -> Result<()> {
        let fd = io::stdin().as_raw_fd();

        // Check if stdin is a TTY
        if unsafe { libc::isatty(fd) } == 0 {
            // Not a TTY - skip raw mode setup
            return Ok(());
        }

        let mut termios = unsafe {
            let mut termios: libc::termios = std::mem::zeroed();
            if libc::tcgetattr(fd, &mut termios) != 0 {
                return Err(Error::Io(io::Error::last_os_error()));
            }
            termios
        };

        self.original_termios = Some(Termios { termios });

        // Set raw mode
        unsafe {
            libc::cfmakeraw(&mut termios);
            if libc::tcsetattr(fd, libc::TCSANOW, &termios) != 0 {
                return Err(Error::Io(io::Error::last_os_error()));
            }
        }

        Ok(())
    }

    #[cfg(unix)]
    fn disable_raw_mode(&mut self) -> Result<()> {
        if let Some(original) = &self.original_termios {
            let fd = io::stdin().as_raw_fd();
            unsafe {
                if libc::tcsetattr(fd, libc::TCSANOW, &original.termios) != 0 {
                    return Err(Error::Io(io::Error::last_os_error()));
                }
            }
        }
        Ok(())
    }

    #[cfg(not(unix))]
    fn enable_raw_mode(&mut self) -> Result<()> {
        // Windows implementation would go here
        Err(Error::NotSupported)
    }

    #[cfg(not(unix))]
    fn disable_raw_mode(&mut self) -> Result<()> {
        Ok(())
    }

    pub(crate) fn read_key_timeout(timeout_ms: Option<u64>) -> Result<Option<Key>> {
        #[cfg(unix)]
        {
            use std::io::ErrorKind;

            let mut buf = [0u8; 8];
            let mut stdin = io::stdin();
            let fd = stdin.as_raw_fd();

            if let Some(timeout) = timeout_ms {
                // Use select to wait for input with timeout
                unsafe {
                    let mut readfds: libc::fd_set = std::mem::zeroed();
                    libc::FD_ZERO(&mut readfds);
                    libc::FD_SET(fd, &mut readfds);

                    let mut tv = libc::timeval {
                        tv_sec: (timeout / 1000) as libc::time_t,
                        tv_usec: ((timeout % 1000) * 1000) as libc::suseconds_t,
                    };

                    let result = libc::select(
                        fd + 1,
                        &mut readfds,
                        std::ptr::null_mut(),
                        std::ptr::null_mut(),
                        &mut tv,
                    );

                    if result == 0 {
                        return Ok(None); // Timeout
                    } else if result < 0 {
                        return Err(Error::Io(io::Error::last_os_error()));
                    }
                }
            }

            // Read available input
            match stdin.read(&mut buf[..1]) {
                Ok(0) => return Ok(None),
                Ok(_) => {
                    let key = Self::parse_key_from_byte(buf[0], &mut stdin, &mut buf)?;
                    return Ok(Some(key));
                }
                Err(e) if e.kind() == ErrorKind::WouldBlock => return Ok(None),
                Err(e) => return Err(e.into()),
            }
        }

        #[cfg(not(unix))]
        {
            Err(Error::NotSupported)
        }
    }

    fn parse_key_from_byte(byte: u8, stdin: &mut io::Stdin, buf: &mut [u8; 8]) -> Result<Key> {
        // Handle special ASCII characters
        match byte {
            b'\r' | b'\n' => return Ok(Key::Enter),
            b'\t' => return Ok(Key::Tab),
            127 => return Ok(Key::Backspace),
            27 => {
                // Escape sequence - try to read more
                let mut seq = vec![27];

                // Use non-blocking read for escape sequences
                #[cfg(unix)]
                {
                    use std::io::ErrorKind;
                    use std::time::Duration;

                    // Set a short timeout to detect lone ESC
                    std::thread::sleep(Duration::from_millis(1));

                    loop {
                        match stdin.read(&mut buf[..1]) {
                            Ok(0) => break,
                            Ok(_) => {
                                seq.push(buf[0]);
                                if seq.len() >= 6 {
                                    break;
                                }
                            }
                            Err(e) if e.kind() == ErrorKind::WouldBlock => break,
                            Err(e) => return Err(e.into()),
                        }
                    }
                }

                if let Some(key) = Key::from_escape_sequence(&seq) {
                    return Ok(key);
                }
                return Ok(Key::Escape);
            }
            1..=26 => {
                // Control characters
                let ch = (byte - 1 + b'a') as char;
                return Ok(Key::Ctrl(ch));
            }
            32..=126 => {
                // Printable ASCII
                return Ok(Key::Char(byte as char));
            }
            _ => return Ok(Key::Unknown),
        }
    }

    pub(crate) fn read_key() -> Result<Key> {
        let mut buf = [0u8; 8];
        let mut stdin = io::stdin();

        let n = stdin.read(&mut buf[..1])?;
        if n == 0 {
            return Ok(Key::Unknown);
        }

        Self::parse_key_from_byte(buf[0], &mut stdin, &mut buf)
    }

    pub(crate) fn get_terminal_size() -> Result<(u16, u16)> {
        #[cfg(unix)]
        {
            let fd = io::stdout().as_raw_fd();

            // Check if stdout is a TTY
            if unsafe { libc::isatty(fd) } == 0 {
                // Not a TTY - return a default size or error
                // For now, return a reasonable default size (24x80 is classic terminal size)
                return Ok((24, 80));
            }

            let mut winsize: libc::winsize = unsafe { std::mem::zeroed() };

            unsafe {
                if libc::ioctl(fd, libc::TIOCGWINSZ, &mut winsize) != 0 {
                    return Err(Error::Io(io::Error::last_os_error()));
                }
            }

            Ok((winsize.ws_row, winsize.ws_col))
        }

        #[cfg(not(unix))]
        {
            Err(Error::NotSupported)
        }
    }

    /// Add content to the update buffer (for wnoutrefresh)
    pub(crate) fn add_to_update_buffer(content: &str) -> Result<()> {
        let buffer = UPDATE_BUFFER.get_or_init(|| Mutex::new(String::new()));
        let mut guard = buffer.lock().unwrap();
        guard.push_str(content);
        Ok(())
    }

    /// Flush the update buffer to screen (doupdate)
    pub(crate) fn doupdate() -> Result<()> {
        let buffer = UPDATE_BUFFER.get_or_init(|| Mutex::new(String::new()));
        let mut guard = buffer.lock().unwrap();

        if !guard.is_empty() {
            io::stdout().write_all(guard.as_bytes())?;
            io::stdout().flush()?;
            guard.clear();
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_backend_creation() {
        let backend = Backend::new();
        assert!(!backend.initialized);
        assert!(backend.original_termios.is_none());
    }

    #[test]
    #[cfg(unix)]
    fn test_terminal_size() {
        // This will work in a real terminal
        if let Ok((rows, cols)) = Backend::get_terminal_size() {
            assert!(rows > 0);
            assert!(cols > 0);
        }
    }
}
