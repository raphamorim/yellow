const std = @import("std");
const yellow = @import("yellow");

// Import image loader wrapper
const c = @cImport({
    @cInclude("image_loader.h");
});

pub fn main() !void {
    const stdout = std.io.getStdOut().writer();

    // Load image using stb_image
    var width: c_int = 0;
    var height: c_int = 0;
    var channels: c_int = 0;

    const img_path = "examples/resources/yellow.png";
    const img_data = c.load_image(img_path, &width, &height, &channels);

    if (img_data == null) {
        try stdout.print("Error: Failed to load image from {s}\n", .{img_path});
        try stdout.print("Make sure the file exists.\n", .{});
        return error.ImageLoadFailed;
    }
    defer c.free_image(img_data);

    const img_size = @as(usize, @intCast(width * height * 3));
    const image_slice = img_data[0..img_size];

    // Render mosaic with threshold 100 and All blocks (matching Rust demo)
    const mosaic = yellow.c.yellow_render_mosaic(
        image_slice.ptr,
        image_slice.len,
        @intCast(width),
        @intCast(height),
        60,  // output width (matching Rust demo)
        100  // threshold (matching Rust demo)
    );
    defer yellow.c.yellow_free_string(mosaic);

    if (mosaic == null) {
        try stdout.print("Error: Failed to render mosaic\n", .{});
        return error.MosaicRenderFailed;
    }

    // Print the mosaic art
    const mosaic_str = std.mem.span(mosaic);
    try stdout.print("{s}\n", .{mosaic_str});
}
