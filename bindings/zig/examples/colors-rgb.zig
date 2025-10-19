const std = @import("std");
const zaz = @import("zaz");

const FpsWidget = struct {
    frame_count: usize,
    last_instant: std.time.Instant,
    fps: ?f32,

    fn init() FpsWidget {
        return .{
            .frame_count = 0,
            .last_instant = std.time.Instant.now() catch unreachable,
            .fps = null,
        };
    }

    fn calculateFps(self: *FpsWidget) void {
        self.frame_count += 1;
        const elapsed_ns = std.time.Instant.now() catch return;
        const elapsed = elapsed_ns.since(self.last_instant);

        if (elapsed > std.time.ns_per_s and self.frame_count > 2) {
            self.fps = @as(f32, @floatFromInt(self.frame_count)) / (@as(f32, @floatFromInt(elapsed)) / @as(f32, @floatFromInt(std.time.ns_per_s)));
            self.frame_count = 0;
            self.last_instant = elapsed_ns;
        }
    }
};

const ColorsWidget = struct {
    colors: std.ArrayList(std.ArrayList(Rgb)),
    frame_count: usize,
    width: usize,
    height: usize,
    allocator: std.mem.Allocator,

    const Rgb = struct {
        r: u8,
        g: u8,
        b: u8,
    };

    fn init(allocator: std.mem.Allocator) ColorsWidget {
        return .{
            .colors = std.ArrayList(std.ArrayList(Rgb)).init(allocator),
            .frame_count = 0,
            .width = 0,
            .height = 0,
            .allocator = allocator,
        };
    }

    fn deinit(self: *ColorsWidget) void {
        for (self.colors.items) |row| {
            row.deinit();
        }
        self.colors.deinit();
    }

    fn hsvToRgb(h: f32, s: f32, v: f32) Rgb {
        const c = v * s;
        const x = c * (1.0 - @abs(@mod(h / 60.0, 2.0) - 1.0));
        const m = v - c;

        var r: f32 = 0;
        var g: f32 = 0;
        var b: f32 = 0;

        if (h >= 0 and h < 60) {
            r = c; g = x; b = 0;
        } else if (h >= 60 and h < 120) {
            r = x; g = c; b = 0;
        } else if (h >= 120 and h < 180) {
            r = 0; g = c; b = x;
        } else if (h >= 180 and h < 240) {
            r = 0; g = x; b = c;
        } else if (h >= 240 and h < 300) {
            r = x; g = 0; b = c;
        } else {
            r = c; g = 0; b = x;
        }

        return .{
            .r = @intFromFloat((r + m) * 255.0),
            .g = @intFromFloat((g + m) * 255.0),
            .b = @intFromFloat((b + m) * 255.0),
        };
    }

    fn setupColors(self: *ColorsWidget, width: u16, height: u16) !void {
        const w = @as(usize, width);
        const h = @as(usize, height) * 2; // Double height for half-blocks

        // Only update if size changed
        if (self.colors.items.len == h and self.width == w) {
            return;
        }

        // Clear existing colors
        for (self.colors.items) |row| {
            row.deinit();
        }
        self.colors.clearRetainingCapacity();

        self.width = w;
        self.height = h;

        // Generate new colors
        var y: usize = 0;
        while (y < h) : (y += 1) {
            var row = std.ArrayList(Rgb).init(self.allocator);
            try row.ensureTotalCapacity(w);

            var x: usize = 0;
            while (x < w) : (x += 1) {
                const hue = @as(f32, @floatFromInt(x)) * 360.0 / @as(f32, @floatFromInt(w));
                const value = @as(f32, @floatFromInt(h - y)) / @as(f32, @floatFromInt(h));
                const saturation: f32 = 1.0;

                const color = hsvToRgb(hue, saturation, value);
                try row.append(color);
            }

            try self.colors.append(row);
        }
    }

    fn render(self: *ColorsWidget, screen: *zaz.Screen, start_row: u16, width: u16) !void {
        const w = @as(usize, width);
        const h = self.height / 2; // Screen rows (each contains 2 pixel rows)

        var y: usize = 0;
        while (y < h) : (y += 1) {
            var x: usize = 0;
            while (x < w) : (x += 1) {
                // Animate by shifting x index
                const xi = (x + self.frame_count) % w;

                // Get colors for top and bottom pixels
                const fg = self.colors.items[y * 2].items[xi];
                const bg = self.colors.items[y * 2 + 1].items[xi];

                // Set colors and draw half block
                try screen.setFgColor(fg.r, fg.g, fg.b);
                try screen.setBgColor(bg.r, bg.g, bg.b);
                try screen.mvprint(@intCast(start_row + y), @intCast(x), "▀");
            }
        }

        self.frame_count += 1;
    }
};

pub fn main() !void {
    var gpa = std.heap.GeneralPurposeAllocator(.{}){};
    defer _ = gpa.deinit();
    const allocator = gpa.allocator();

    const screen = try zaz.Screen.init();
    defer screen.deinit() catch {};

    var fps_widget = FpsWidget.init();
    var colors_widget = ColorsWidget.init(allocator);
    defer colors_widget.deinit();

    var running = true;
    while (running) {
        const size = screen.getSize();
        const rows = size.height;
        const cols = size.width;

        // Clear screen
        try screen.clear();

        // Render top bar with title and FPS
        const separator = "─";
        var sep_buf: [256]u8 = undefined;
        var sep_line = try std.fmt.bufPrint(&sep_buf, "{s}" ** cols, .{separator} ** cols);
        try screen.mvprint(1, 0, sep_line[0..cols]);

        // Render title (centered)
        const title = "colors_rgb example. Press any key to quit";
        const title_x = @divTrunc(cols, 2) - @divTrunc(@as(u16, @intCast(title.len)), 2);
        try screen.mvprint(0, title_x, title);

        // Render FPS in top left with styling
        fps_widget.calculateFps();
        if (fps_widget.fps) |fps| {
            try screen.attrOn(.bold);
            try screen.setFgColor(100, 200, 255);
            var fps_buf: [32]u8 = undefined;
            const fps_text = try std.fmt.bufPrint(&fps_buf, "FPS: {d:.1}", .{fps});
            try screen.mvprint(0, 0, fps_text);
            try screen.attrOff(.bold);
            try screen.setFgColor(255, 255, 255);
        }

        // Render colors widget (starting from row 2, after separator)
        const colors_height = if (rows > 2) rows - 2 else 0;
        try colors_widget.setupColors(cols, colors_height);
        try colors_widget.render(screen, 2, cols);

        try screen.refresh();

        // Check for input with timeout (~60 FPS)
        if (try screen.getchTimeout(16)) |_| {
            running = false;
        }
    }
}
