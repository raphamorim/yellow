# colors-rgb-ncurses

An ncurses/C implementation of the colors-rgb example, demonstrating the full range of RGB colors that can be displayed in a terminal.

## Features

- **24-bit True Color Support**: Displays the full RGB color spectrum using ANSI escape sequences
- **Animated Color Gradient**: Smoothly scrolling rainbow gradient arranged by hue (horizontal) and brightness (vertical)
- **High Resolution Display**: Uses Unicode half-block characters (▀) to render two "pixels" per terminal cell
- **FPS Counter**: Real-time frames-per-second display
- **Smooth Animation**: Targets ~60 FPS with double-buffering

## Requirements

- A terminal emulator that supports:
  - 24-bit true color (most modern terminals: iTerm2, Alacritty, kitty, Windows Terminal, etc.)
  - UTF-8/Unicode characters
- ncurses library
- C compiler (gcc or clang)

## Building

```bash
make
```

Or manually:

```bash
gcc -Wall -Wextra -O2 -std=c99 -o colors-rgb colors-rgb.c -lncurses -lm
```

## Running

```bash
make run
```

Or:

```bash
./colors-rgb
```

Press any key to quit.

## How It Works

The example creates a color gradient using HSV color space:
- **Horizontal axis (X)**: Hue (0-360°) - cycles through all colors of the rainbow
- **Vertical axis (Y)**: Value/Brightness (0-100%) - from dark at bottom to bright at top
- **Saturation**: Fixed at maximum (100%) for vibrant colors

Each terminal cell displays two color pixels using the half-block character (▀):
- **Foreground color**: Top pixel
- **Background color**: Bottom pixel

This technique doubles the vertical resolution of the color display.

## Comparison with Rust Version

This C/ncurses implementation mirrors the functionality of `examples/apps/colors-rgb` written in Rust using the Zaz library. Both versions:
- Display the same HSV-based color gradient
- Animate by scrolling colors horizontally
- Show FPS in the top-right corner
- Use half-block characters for higher resolution
- Target ~60 FPS performance

## Notes

If your terminal doesn't support true color, you may see garbled output or incorrect colors. Check your terminal's documentation for true color support.
