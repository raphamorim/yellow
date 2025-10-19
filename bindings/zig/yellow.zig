const std = @import("std");

// Import the C header
pub const c = @cImport({
    @cInclude("zaz.h");
});

/// Opaque screen handle
pub const Screen = opaque {
    /// Initialize a new Zaz screen
    pub fn init() !*Screen {
        const screen = c.zaz_init();
        if (screen == null) {
            return error.InitFailed;
        }
        return @ptrCast(screen);
    }

    /// Clean up and restore terminal
    pub fn deinit(self: *Screen) !void {
        const result = c.zaz_endwin(@ptrCast(self));
        if (result != 0) {
            return error.EndwinFailed;
        }
    }

    /// Clear the screen
    pub fn clear(self: *Screen) !void {
        const result = c.zaz_clear(@ptrCast(self));
        if (result != 0) {
            return error.ClearFailed;
        }
    }

    /// Refresh the screen (flush output)
    pub fn refresh(self: *Screen) !void {
        const result = c.zaz_refresh(@ptrCast(self));
        if (result != 0) {
            return error.RefreshFailed;
        }
    }

    /// Move cursor to position (y, x)
    pub fn moveCursor(self: *Screen, y: u16, x: u16) !void {
        const result = c.zaz_move_cursor(@ptrCast(self), y, x);
        if (result != 0) {
            return error.MoveCursorFailed;
        }
    }

    /// Print string at current cursor position
    pub fn print(self: *Screen, text: []const u8) !void {
        const c_str = @as([*c]const u8, @ptrCast(text.ptr));
        const result = c.zaz_print(@ptrCast(self), c_str);
        if (result != 0) {
            return error.PrintFailed;
        }
    }

    /// Print string at position (y, x)
    pub fn mvprint(self: *Screen, y: u16, x: u16, text: []const u8) !void {
        const c_str = @as([*c]const u8, @ptrCast(text.ptr));
        const result = c.zaz_mvprint(@ptrCast(self), y, x, c_str);
        if (result != 0) {
            return error.MvprintFailed;
        }
    }

    /// Get a key from input
    pub fn getch(self: *Screen) !Key {
        var c_key: c.ZazKey = undefined;
        const result = c.zaz_getch(@ptrCast(self), &c_key);
        if (result != 0) {
            return error.GetchFailed;
        }
        return Key.fromC(c_key);
    }

    /// Set foreground color (RGB)
    pub fn setFgColor(self: *Screen, r: u8, g: u8, b: u8) !void {
        const result = c.zaz_set_fg_color(@ptrCast(self), r, g, b);
        if (result != 0) {
            return error.SetColorFailed;
        }
    }

    /// Set background color (RGB)
    pub fn setBgColor(self: *Screen, r: u8, g: u8, b: u8) !void {
        const result = c.zaz_set_bg_color(@ptrCast(self), r, g, b);
        if (result != 0) {
            return error.SetColorFailed;
        }
    }

    /// Turn on attribute
    pub fn attrOn(self: *Screen, attr: Attr) !void {
        const result = c.zaz_attron(@ptrCast(self), @intFromEnum(attr));
        if (result != 0) {
            return error.AttrFailed;
        }
    }

    /// Turn off attribute
    pub fn attrOff(self: *Screen, attr: Attr) !void {
        const result = c.zaz_attroff(@ptrCast(self), @intFromEnum(attr));
        if (result != 0) {
            return error.AttrFailed;
        }
    }

    /// Get terminal size as (height, width)
    pub fn getSize(self: *Screen) struct { height: u16, width: u16 } {
        const size = c.zaz_get_size(@ptrCast(self));
        return .{
            .height = @intCast(size >> 16),
            .width = @intCast(size & 0xFFFF),
        };
    }
};

/// Key events
pub const Key = union(enum) {
    char: u32,
    arrow_up,
    arrow_down,
    arrow_left,
    arrow_right,
    enter,
    backspace,
    delete,
    home,
    end,
    page_up,
    page_down,
    tab,
    escape,
    f1,
    f2,
    f3,
    f4,
    f5,
    f6,
    f7,
    f8,
    f9,
    f10,
    f11,
    f12,
    unknown,

    fn fromC(c_key: c.ZazKey) Key {
        return switch (c_key.tag) {
            c.ZazKey_Char => .{ .char = c_key.value.char_value },
            c.ZazKey_ArrowUp => .arrow_up,
            c.ZazKey_ArrowDown => .arrow_down,
            c.ZazKey_ArrowLeft => .arrow_left,
            c.ZazKey_ArrowRight => .arrow_right,
            c.ZazKey_Enter => .enter,
            c.ZazKey_Backspace => .backspace,
            c.ZazKey_Delete => .delete,
            c.ZazKey_Home => .home,
            c.ZazKey_End => .end,
            c.ZazKey_PageUp => .page_up,
            c.ZazKey_PageDown => .page_down,
            c.ZazKey_Tab => .tab,
            c.ZazKey_Escape => .escape,
            c.ZazKey_F1 => .f1,
            c.ZazKey_F2 => .f2,
            c.ZazKey_F3 => .f3,
            c.ZazKey_F4 => .f4,
            c.ZazKey_F5 => .f5,
            c.ZazKey_F6 => .f6,
            c.ZazKey_F7 => .f7,
            c.ZazKey_F8 => .f8,
            c.ZazKey_F9 => .f9,
            c.ZazKey_F10 => .f10,
            c.ZazKey_F11 => .f11,
            c.ZazKey_F12 => .f12,
            c.ZazKey_Unknown => .unknown,
            else => .unknown,
        };
    }
};

/// Text attributes
pub const Attr = enum(u32) {
    bold = c.YELLOW_ATTR_BOLD,
    dim = c.YELLOW_ATTR_DIM,
    italic = c.YELLOW_ATTR_ITALIC,
    underline = c.YELLOW_ATTR_UNDERLINE,
    blink = c.YELLOW_ATTR_BLINK,
    reverse = c.YELLOW_ATTR_REVERSE,
    strikethrough = c.YELLOW_ATTR_STRIKETHROUGH,
};
