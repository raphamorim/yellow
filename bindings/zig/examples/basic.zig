const std = @import("std");
const zaz = @import("zaz");

pub fn main() !void {
    // Initialize screen
    const screen = try zaz.Screen.init();
    defer screen.deinit() catch {};

    // Clear and draw
    try screen.clear();

    // Print welcome message
    try screen.mvprint(2, 4, "Hello from Zig + Zaz!");

    // Set colors and print
    try screen.setFgColor(255, 200, 0);
    try screen.attrOn(.bold);
    try screen.mvprint(4, 4, "Zaz bindings working!");
    try screen.attrOff(.bold);

    // Get terminal size
    const size = screen.getSize();
    var buf: [100]u8 = undefined;
    const msg = try std.fmt.bufPrint(&buf, "Terminal size: {}x{}", .{ size.height, size.width });
    try screen.mvprint(6, 4, msg);

    // Instructions
    try screen.setFgColor(150, 150, 150);
    try screen.mvprint(8, 4, "Press any key to exit...");

    try screen.refresh();

    // Wait for key
    _ = try screen.getch();
}
