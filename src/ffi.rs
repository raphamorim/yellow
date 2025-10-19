//! C FFI bindings for Zaz library
//!
//! This module provides C-compatible exports for use with other languages.

use std::ffi::CStr;
use std::os::raw::c_char;
use std::ptr;

use crate::{Attr, Color, Key, Screen};

/// Opaque handle to a Screen
#[repr(C)]
pub struct ZazScreen {
    _private: [u8; 0],
}

/// Key tag for discriminated union
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub enum ZazKeyTag {
    Char = 0,
    ArrowUp,
    ArrowDown,
    ArrowLeft,
    ArrowRight,
    Enter,
    Backspace,
    Delete,
    Home,
    End,
    PageUp,
    PageDown,
    Tab,
    Escape,
    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,
    Unknown,
}

/// Union for key value
#[repr(C)]
#[derive(Copy, Clone)]
pub union ZazKeyValue {
    pub char_value: u32,
}

/// Key codes for input (tagged union)
#[repr(C)]
#[derive(Copy, Clone)]
pub struct ZazKey {
    pub tag: ZazKeyTag,
    pub value: ZazKeyValue,
}

impl From<Key> for ZazKey {
    fn from(key: Key) -> Self {
        match key {
            Key::Char(c) => ZazKey {
                tag: ZazKeyTag::Char,
                value: ZazKeyValue {
                    char_value: c as u32,
                },
            },
            Key::Up => ZazKey {
                tag: ZazKeyTag::ArrowUp,
                value: ZazKeyValue { char_value: 0 },
            },
            Key::Down => ZazKey {
                tag: ZazKeyTag::ArrowDown,
                value: ZazKeyValue { char_value: 0 },
            },
            Key::Left => ZazKey {
                tag: ZazKeyTag::ArrowLeft,
                value: ZazKeyValue { char_value: 0 },
            },
            Key::Right => ZazKey {
                tag: ZazKeyTag::ArrowRight,
                value: ZazKeyValue { char_value: 0 },
            },
            Key::Enter => ZazKey {
                tag: ZazKeyTag::Enter,
                value: ZazKeyValue { char_value: 0 },
            },
            Key::Backspace => ZazKey {
                tag: ZazKeyTag::Backspace,
                value: ZazKeyValue { char_value: 0 },
            },
            Key::Delete => ZazKey {
                tag: ZazKeyTag::Delete,
                value: ZazKeyValue { char_value: 0 },
            },
            Key::Home => ZazKey {
                tag: ZazKeyTag::Home,
                value: ZazKeyValue { char_value: 0 },
            },
            Key::End => ZazKey {
                tag: ZazKeyTag::End,
                value: ZazKeyValue { char_value: 0 },
            },
            Key::PageUp => ZazKey {
                tag: ZazKeyTag::PageUp,
                value: ZazKeyValue { char_value: 0 },
            },
            Key::PageDown => ZazKey {
                tag: ZazKeyTag::PageDown,
                value: ZazKeyValue { char_value: 0 },
            },
            Key::Tab => ZazKey {
                tag: ZazKeyTag::Tab,
                value: ZazKeyValue { char_value: 0 },
            },
            Key::Escape => ZazKey {
                tag: ZazKeyTag::Escape,
                value: ZazKeyValue { char_value: 0 },
            },
            Key::F(1) => ZazKey {
                tag: ZazKeyTag::F1,
                value: ZazKeyValue { char_value: 0 },
            },
            Key::F(2) => ZazKey {
                tag: ZazKeyTag::F2,
                value: ZazKeyValue { char_value: 0 },
            },
            Key::F(3) => ZazKey {
                tag: ZazKeyTag::F3,
                value: ZazKeyValue { char_value: 0 },
            },
            Key::F(4) => ZazKey {
                tag: ZazKeyTag::F4,
                value: ZazKeyValue { char_value: 0 },
            },
            Key::F(5) => ZazKey {
                tag: ZazKeyTag::F5,
                value: ZazKeyValue { char_value: 0 },
            },
            Key::F(6) => ZazKey {
                tag: ZazKeyTag::F6,
                value: ZazKeyValue { char_value: 0 },
            },
            Key::F(7) => ZazKey {
                tag: ZazKeyTag::F7,
                value: ZazKeyValue { char_value: 0 },
            },
            Key::F(8) => ZazKey {
                tag: ZazKeyTag::F8,
                value: ZazKeyValue { char_value: 0 },
            },
            Key::F(9) => ZazKey {
                tag: ZazKeyTag::F9,
                value: ZazKeyValue { char_value: 0 },
            },
            Key::F(10) => ZazKey {
                tag: ZazKeyTag::F10,
                value: ZazKeyValue { char_value: 0 },
            },
            Key::F(11) => ZazKey {
                tag: ZazKeyTag::F11,
                value: ZazKeyValue { char_value: 0 },
            },
            Key::F(12) => ZazKey {
                tag: ZazKeyTag::F12,
                value: ZazKeyValue { char_value: 0 },
            },
            _ => ZazKey {
                tag: ZazKeyTag::Unknown,
                value: ZazKeyValue { char_value: 0 },
            },
        }
    }
}

/// Initialize a new screen
///
/// Returns NULL on error
#[unsafe(no_mangle)]
pub extern "C" fn zaz_init() -> *mut ZazScreen {
    match Screen::init() {
        Ok(screen) => Box::into_raw(Box::new(screen)) as *mut ZazScreen,
        Err(_) => ptr::null_mut(),
    }
}

/// Clean up and restore terminal
#[unsafe(no_mangle)]
pub extern "C" fn zaz_endwin(screen: *mut ZazScreen) -> i32 {
    if screen.is_null() {
        return -1;
    }

    unsafe {
        let screen = Box::from_raw(screen as *mut Screen);
        match screen.endwin() {
            Ok(_) => 0,
            Err(_) => -1,
        }
    }
}

/// Clear the screen
#[unsafe(no_mangle)]
pub extern "C" fn zaz_clear(screen: *mut ZazScreen) -> i32 {
    if screen.is_null() {
        return -1;
    }

    unsafe {
        let screen = &mut *(screen as *mut Screen);
        match screen.clear() {
            Ok(_) => 0,
            Err(_) => -1,
        }
    }
}

/// Refresh the screen (flush output)
#[unsafe(no_mangle)]
pub extern "C" fn zaz_refresh(screen: *mut ZazScreen) -> i32 {
    if screen.is_null() {
        return -1;
    }

    unsafe {
        let screen = &mut *(screen as *mut Screen);
        match screen.refresh() {
            Ok(_) => 0,
            Err(_) => -1,
        }
    }
}

/// Move cursor to position (y, x)
#[unsafe(no_mangle)]
pub extern "C" fn zaz_move_cursor(screen: *mut ZazScreen, y: u16, x: u16) -> i32 {
    if screen.is_null() {
        return -1;
    }

    unsafe {
        let screen = &mut *(screen as *mut Screen);
        match screen.move_cursor(y, x) {
            Ok(_) => 0,
            Err(_) => -1,
        }
    }
}

/// Print string at current cursor position
#[unsafe(no_mangle)]
pub extern "C" fn zaz_print(screen: *mut ZazScreen, text: *const c_char) -> i32 {
    if screen.is_null() || text.is_null() {
        return -1;
    }

    unsafe {
        let screen = &mut *(screen as *mut Screen);
        let c_str = CStr::from_ptr(text);

        match c_str.to_str() {
            Ok(s) => match screen.print(s) {
                Ok(_) => 0,
                Err(_) => -1,
            },
            Err(_) => -1,
        }
    }
}

/// Print string at position (y, x)
#[unsafe(no_mangle)]
pub extern "C" fn zaz_mvprint(screen: *mut ZazScreen, y: u16, x: u16, text: *const c_char) -> i32 {
    if screen.is_null() || text.is_null() {
        return -1;
    }

    unsafe {
        let screen = &mut *(screen as *mut Screen);
        let c_str = CStr::from_ptr(text);

        match c_str.to_str() {
            Ok(s) => match screen.mvprint(y, x, s) {
                Ok(_) => 0,
                Err(_) => -1,
            },
            Err(_) => -1,
        }
    }
}

/// Get a key from input
#[unsafe(no_mangle)]
pub extern "C" fn zaz_getch(screen: *mut ZazScreen, key_out: *mut ZazKey) -> i32 {
    if screen.is_null() || key_out.is_null() {
        return -1;
    }

    unsafe {
        let screen = &mut *(screen as *mut Screen);
        match screen.getch() {
            Ok(key) => {
                *key_out = key.into();
                0
            }
            Err(_) => -1,
        }
    }
}

/// Get a key from input with timeout
/// Returns 1 if key was pressed (key_out is set), 0 if timeout, -1 on error
#[unsafe(no_mangle)]
pub extern "C" fn zaz_getch_timeout(
    screen: *mut ZazScreen,
    timeout_ms: u64,
    key_out: *mut ZazKey,
) -> i32 {
    if screen.is_null() || key_out.is_null() {
        return -1;
    }

    unsafe {
        let screen = &mut *(screen as *mut Screen);
        match screen.getch_timeout(timeout_ms) {
            Ok(Some(key)) => {
                *key_out = key.into();
                1 // Key was pressed
            }
            Ok(None) => 0, // Timeout
            Err(_) => -1,  // Error
        }
    }
}

/// Set foreground color
#[unsafe(no_mangle)]
pub extern "C" fn zaz_set_fg_color(screen: *mut ZazScreen, r: u8, g: u8, b: u8) -> i32 {
    if screen.is_null() {
        return -1;
    }

    unsafe {
        let screen = &mut *(screen as *mut Screen);
        let color = Color::Rgb(r, g, b);
        match screen.set_fg(color) {
            Ok(_) => 0,
            Err(_) => -1,
        }
    }
}

/// Set background color
#[unsafe(no_mangle)]
pub extern "C" fn zaz_set_bg_color(screen: *mut ZazScreen, r: u8, g: u8, b: u8) -> i32 {
    if screen.is_null() {
        return -1;
    }

    unsafe {
        let screen = &mut *(screen as *mut Screen);
        let color = Color::Rgb(r, g, b);
        match screen.set_bg(color) {
            Ok(_) => 0,
            Err(_) => -1,
        }
    }
}

/// Turn on attribute (BOLD=1, DIM=2, ITALIC=4, UNDERLINE=8, BLINK=16, REVERSE=32, STRIKETHROUGH=128)
#[unsafe(no_mangle)]
pub extern "C" fn zaz_attron(screen: *mut ZazScreen, attr: u32) -> i32 {
    if screen.is_null() {
        return -1;
    }

    unsafe {
        let screen = &mut *(screen as *mut Screen);
        let attr = Attr(attr as u16);
        match screen.attron(attr) {
            Ok(_) => 0,
            Err(_) => -1,
        }
    }
}

/// Turn off attribute
#[unsafe(no_mangle)]
pub extern "C" fn zaz_attroff(screen: *mut ZazScreen, attr: u32) -> i32 {
    if screen.is_null() {
        return -1;
    }

    unsafe {
        let screen = &mut *(screen as *mut Screen);
        let attr = Attr(attr as u16);
        match screen.attroff(attr) {
            Ok(_) => 0,
            Err(_) => -1,
        }
    }
}

/// Get terminal size (returns height in high 16 bits, width in low 16 bits, or 0 on error)
#[unsafe(no_mangle)]
pub extern "C" fn zaz_get_size(screen: *mut ZazScreen) -> u32 {
    if screen.is_null() {
        return 0;
    }

    unsafe {
        let screen = &*(screen as *mut Screen);
        match screen.get_size() {
            Ok((height, width)) => ((height as u32) << 16) | (width as u32),
            Err(_) => 0,
        }
    }
}

/// Render mosaic (Unicode block art) from RGB image data
///
/// Returns a malloc'd C string that must be freed by the caller
/// Returns NULL on error
#[unsafe(no_mangle)]
pub extern "C" fn zaz_render_mosaic(
    data: *const u8,
    data_len: usize,
    width: usize,
    height: usize,
    output_width: usize,
    threshold: u8,
) -> *mut i8 {
    if data.is_null() || data_len == 0 {
        return ptr::null_mut();
    }

    unsafe {
        let slice = std::slice::from_raw_parts(data, data_len);

        let config = crate::MosaicConfig::with_width(output_width).threshold(threshold);

        let result = crate::render_mosaic(slice, width, height, &config);

        // Convert to C string
        match std::ffi::CString::new(result) {
            Ok(c_str) => c_str.into_raw(),
            Err(_) => ptr::null_mut(),
        }
    }
}

/// Free a string returned by zaz_render_mosaic
#[unsafe(no_mangle)]
pub extern "C" fn zaz_free_string(s: *mut i8) {
    if !s.is_null() {
        unsafe {
            let _ = std::ffi::CString::from_raw(s);
        }
    }
}

/// Constants for attributes
pub const YELLOW_ATTR_BOLD: u32 = 1;
pub const YELLOW_ATTR_DIM: u32 = 2;
pub const YELLOW_ATTR_ITALIC: u32 = 4;
pub const YELLOW_ATTR_UNDERLINE: u32 = 8;
pub const YELLOW_ATTR_BLINK: u32 = 16;
pub const YELLOW_ATTR_REVERSE: u32 = 32;
pub const YELLOW_ATTR_HIDDEN: u32 = 64;
pub const YELLOW_ATTR_STRIKETHROUGH: u32 = 128;
