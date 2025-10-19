//! C FFI bindings for Yellow library
//!
//! This module provides C-compatible exports for use with other languages.

use std::ffi::CStr;
use std::os::raw::c_char;
use std::ptr;

use crate::{Attr, Color, Key, Screen};

/// Opaque handle to a Screen
#[repr(C)]
pub struct YellowScreen {
    _private: [u8; 0],
}

/// Key tag for discriminated union
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub enum YellowKeyTag {
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
pub union YellowKeyValue {
    pub char_value: u32,
}

/// Key codes for input (tagged union)
#[repr(C)]
#[derive(Copy, Clone)]
pub struct YellowKey {
    pub tag: YellowKeyTag,
    pub value: YellowKeyValue,
}

impl From<Key> for YellowKey {
    fn from(key: Key) -> Self {
        match key {
            Key::Char(c) => YellowKey {
                tag: YellowKeyTag::Char,
                value: YellowKeyValue {
                    char_value: c as u32,
                },
            },
            Key::Up => YellowKey {
                tag: YellowKeyTag::ArrowUp,
                value: YellowKeyValue { char_value: 0 },
            },
            Key::Down => YellowKey {
                tag: YellowKeyTag::ArrowDown,
                value: YellowKeyValue { char_value: 0 },
            },
            Key::Left => YellowKey {
                tag: YellowKeyTag::ArrowLeft,
                value: YellowKeyValue { char_value: 0 },
            },
            Key::Right => YellowKey {
                tag: YellowKeyTag::ArrowRight,
                value: YellowKeyValue { char_value: 0 },
            },
            Key::Enter => YellowKey {
                tag: YellowKeyTag::Enter,
                value: YellowKeyValue { char_value: 0 },
            },
            Key::Backspace => YellowKey {
                tag: YellowKeyTag::Backspace,
                value: YellowKeyValue { char_value: 0 },
            },
            Key::Delete => YellowKey {
                tag: YellowKeyTag::Delete,
                value: YellowKeyValue { char_value: 0 },
            },
            Key::Home => YellowKey {
                tag: YellowKeyTag::Home,
                value: YellowKeyValue { char_value: 0 },
            },
            Key::End => YellowKey {
                tag: YellowKeyTag::End,
                value: YellowKeyValue { char_value: 0 },
            },
            Key::PageUp => YellowKey {
                tag: YellowKeyTag::PageUp,
                value: YellowKeyValue { char_value: 0 },
            },
            Key::PageDown => YellowKey {
                tag: YellowKeyTag::PageDown,
                value: YellowKeyValue { char_value: 0 },
            },
            Key::Tab => YellowKey {
                tag: YellowKeyTag::Tab,
                value: YellowKeyValue { char_value: 0 },
            },
            Key::Escape => YellowKey {
                tag: YellowKeyTag::Escape,
                value: YellowKeyValue { char_value: 0 },
            },
            Key::F(1) => YellowKey {
                tag: YellowKeyTag::F1,
                value: YellowKeyValue { char_value: 0 },
            },
            Key::F(2) => YellowKey {
                tag: YellowKeyTag::F2,
                value: YellowKeyValue { char_value: 0 },
            },
            Key::F(3) => YellowKey {
                tag: YellowKeyTag::F3,
                value: YellowKeyValue { char_value: 0 },
            },
            Key::F(4) => YellowKey {
                tag: YellowKeyTag::F4,
                value: YellowKeyValue { char_value: 0 },
            },
            Key::F(5) => YellowKey {
                tag: YellowKeyTag::F5,
                value: YellowKeyValue { char_value: 0 },
            },
            Key::F(6) => YellowKey {
                tag: YellowKeyTag::F6,
                value: YellowKeyValue { char_value: 0 },
            },
            Key::F(7) => YellowKey {
                tag: YellowKeyTag::F7,
                value: YellowKeyValue { char_value: 0 },
            },
            Key::F(8) => YellowKey {
                tag: YellowKeyTag::F8,
                value: YellowKeyValue { char_value: 0 },
            },
            Key::F(9) => YellowKey {
                tag: YellowKeyTag::F9,
                value: YellowKeyValue { char_value: 0 },
            },
            Key::F(10) => YellowKey {
                tag: YellowKeyTag::F10,
                value: YellowKeyValue { char_value: 0 },
            },
            Key::F(11) => YellowKey {
                tag: YellowKeyTag::F11,
                value: YellowKeyValue { char_value: 0 },
            },
            Key::F(12) => YellowKey {
                tag: YellowKeyTag::F12,
                value: YellowKeyValue { char_value: 0 },
            },
            _ => YellowKey {
                tag: YellowKeyTag::Unknown,
                value: YellowKeyValue { char_value: 0 },
            },
        }
    }
}

/// Initialize a new screen
///
/// Returns NULL on error
#[unsafe(no_mangle)]
pub extern "C" fn yellow_init() -> *mut YellowScreen {
    match Screen::init() {
        Ok(screen) => Box::into_raw(Box::new(screen)) as *mut YellowScreen,
        Err(_) => ptr::null_mut(),
    }
}

/// Clean up and restore terminal
#[unsafe(no_mangle)]
pub extern "C" fn yellow_endwin(screen: *mut YellowScreen) -> i32 {
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
pub extern "C" fn yellow_clear(screen: *mut YellowScreen) -> i32 {
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
pub extern "C" fn yellow_refresh(screen: *mut YellowScreen) -> i32 {
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
pub extern "C" fn yellow_move_cursor(screen: *mut YellowScreen, y: u16, x: u16) -> i32 {
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
pub extern "C" fn yellow_print(screen: *mut YellowScreen, text: *const c_char) -> i32 {
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
pub extern "C" fn yellow_mvprint(
    screen: *mut YellowScreen,
    y: u16,
    x: u16,
    text: *const c_char,
) -> i32 {
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
pub extern "C" fn yellow_getch(screen: *mut YellowScreen, key_out: *mut YellowKey) -> i32 {
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

/// Set foreground color
#[unsafe(no_mangle)]
pub extern "C" fn yellow_set_fg_color(screen: *mut YellowScreen, r: u8, g: u8, b: u8) -> i32 {
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
pub extern "C" fn yellow_set_bg_color(screen: *mut YellowScreen, r: u8, g: u8, b: u8) -> i32 {
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
pub extern "C" fn yellow_attron(screen: *mut YellowScreen, attr: u32) -> i32 {
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
pub extern "C" fn yellow_attroff(screen: *mut YellowScreen, attr: u32) -> i32 {
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
pub extern "C" fn yellow_get_size(screen: *mut YellowScreen) -> u32 {
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
pub extern "C" fn yellow_render_mosaic(
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

/// Free a string returned by yellow_render_mosaic
#[unsafe(no_mangle)]
pub extern "C" fn yellow_free_string(s: *mut i8) {
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
