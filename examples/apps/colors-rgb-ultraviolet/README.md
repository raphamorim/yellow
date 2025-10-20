# Colors-RGB Demo (Ultraviolet)

This example shows the full range of RGB colors in an animated gradient, built using [ultraviolet](https://github.com/charmbracelet/ultraviolet).

Requires a terminal that supports 24-bit color (true color) and unicode.

## Features

- RGB color rendering using ultraviolet's primitives
- Double-buffering for smooth animation
- FPS calculation and display
- Half-block characters for higher resolution color display
- Animated color gradient that shifts horizontally

## Running

To run this demo:

```shell
cd examples/apps/colors-rgb-ultraviolet
make run
```

Or directly with Go:

```shell
go run .
```

Press any key to quit.

## About Ultraviolet

Ultraviolet is a Go library providing primitives for manipulating terminal emulators, with a focus on terminal user interfaces (TUIs). It powers critical portions of Bubble Tea v2 and Lip Gloss v2.

Key features used in this demo:
- Cell-based rendering with diffing algorithm
- True color (24-bit RGB) support
- Cross-platform terminal handling
