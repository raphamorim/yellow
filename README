# Yellow

A terminal manipulation library for Rust with ncurses-inspired API and C/FFI bindings for other languages.

-![Yellow's mosaic demo](examples/resources/yellow.png)

Yellow's mosaic demo with Zig wrapper

-![Yellow's mosaic demo with Zig wrapper](examples/resources/demo-bindings-zig.png)

## Features

- Terminal initialization and screen management
- Cursor positioning and text output
- RGB color support with ANSI escape codes
- Text attributes (bold, italic, underline, etc.)
- Window and panel management
- Keyboard input handling with Kitty keyboard protocol
- Graphics support (Kitty image protocol, Sixel)
- Unicode block mosaic rendering from images
- Scrolling regions

## Installation

### Rust

Add to your `Cargo.toml`:

```toml
[dependencies]
yellow = "*"
```

### Zig

To use Yellow in your Zig project:

1. Build the Yellow library:
```bash
cargo build --release
```

2. Copy the necessary files to your project:
```bash
cp bindings/zig/yellow.zig your-project/
cp bindings/zig/yellow.h your-project/
cp target/release/libyellow.dylib your-project/  # macOS
# or
cp target/release/libyellow.so your-project/     # Linux
```

3. In your `build.zig`:
```zig
const yellow_mod = b.createModule(.{
    .root_source_file = b.path("yellow.zig"),
    .link_libc = true,
});
yellow_mod.addIncludePath(b.path("."));

const exe = b.addExecutable(.{
    .name = "my-app",
    .root_source_file = b.path("main.zig"),
    .target = target,
    .optimize = optimize,
});
exe.root_module.addImport("yellow", yellow_mod);
exe.addIncludePath(b.path("."));
exe.addLibraryPath(b.path("."));
exe.linkSystemLibrary("yellow");
exe.linkLibC();
```

4. Run with library path:
```bash
# macOS
DYLD_LIBRARY_PATH=. zig build run

# Linux
LD_LIBRARY_PATH=. zig build run
```

## Usage

### Rust Example

```rust
use yellow::{Screen, Color, Attr};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut scr = Screen::init()?;

    scr.clear()?;
    scr.mvprint(2, 4, "Hello from Yellow!")?;

    scr.set_fg(Color::Rgb(255, 200, 0))?;
    scr.attron(Attr::BOLD)?;
    scr.mvprint(4, 4, "Colored text!")?;

    scr.refresh()?;
    scr.getch()?;
    scr.endwin()?;

    Ok(())
}
```

### Zig Example

```zig
const std = @import("std");
const yellow = @import("yellow");

pub fn main() !void {
    const screen = try yellow.Screen.init();
    defer screen.deinit() catch {};

    try screen.clear();
    try screen.mvprint(2, 4, "Hello from Zig + Yellow!");

    try screen.setFgColor(255, 200, 0);
    try screen.attrOn(.bold);
    try screen.mvprint(4, 4, "Colored text!");
    try screen.attrOff(.bold);

    try screen.refresh();
    _ = try screen.getch();
}
```

## Building

### Build Rust Library

```bash
cargo build --release
```

### Build Zig Bindings

```bash
make zig-example
```

Or build everything at once:

```bash
make
```

## Running Examples

### Rust Examples

```bash
# Basic demo
cargo run --example demo

# Mosaic rendering from image
cargo run --example mosaic
```

### Zig Examples

```bash
# Basic example
make run-zig

# Or run directly:
DYLD_LIBRARY_PATH=target/release ./bindings/zig/zig-out/bin/basic

# Mosaic example (loads and displays yellow.png as Unicode art)
make run-zig-mosaic

# Or run directly:
DYLD_LIBRARY_PATH=target/release ./bindings/zig/zig-out/bin/mosaic
```

On Linux, use `LD_LIBRARY_PATH` instead of `DYLD_LIBRARY_PATH`.

## Mosaic Rendering

Yellow includes a mosaic module that converts images to Unicode block art with ANSI colors.

Example:

```rust
use yellow::{render_mosaic, MosaicConfig};

let img = image::open("examples/resources/yellow.png")?;
let rgb_img = img.to_rgb8();
let (width, height) = rgb_img.dimensions();
let data = rgb_img.as_raw();

let config = MosaicConfig::with_width(60).threshold(100);
let art = render_mosaic(data, width as usize, height as usize, &config);
println!("{}", art);
```

The same functionality is available through the C FFI:

```c
char* mosaic = yellow_render_mosaic(data, data_len, width, height, 60, 100);
printf("%s\n", mosaic);
yellow_free_string(mosaic);
```

## C FFI API

The library exports a C-compatible API for use with other languages:

### Screen Management
- `yellow_init()` - Initialize screen
- `yellow_endwin()` - Clean up and restore terminal
- `yellow_clear()` - Clear screen
- `yellow_refresh()` - Refresh display

### Output
- `yellow_print()` - Print at cursor position
- `yellow_mvprint()` - Print at specific position
- `yellow_move_cursor()` - Move cursor

### Colors and Attributes
- `yellow_set_fg_color()` - Set foreground RGB color
- `yellow_set_bg_color()` - Set background RGB color
- `yellow_attron()` - Enable text attributes
- `yellow_attroff()` - Disable text attributes

### Input
- `yellow_getch()` - Get key input

### Utilities
- `yellow_get_size()` - Get terminal dimensions
- `yellow_render_mosaic()` - Render image as Unicode art
- `yellow_free_string()` - Free mosaic string

## License

See LICENSE file for details.
