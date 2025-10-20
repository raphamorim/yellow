// A demo application that shows the full range of RGB colors that can be displayed in the terminal.
//
// Requires a terminal that supports 24-bit color (true color) and unicode.
//
// This example demonstrates:
// - RGB color rendering with ultraviolet
// - Double-buffering for smooth animation
// - FPS calculation and display
// - Using half-block characters for higher resolution color display
//
// Press q to quit.

package main

import (
	"context"
	"fmt"
	"image/color"
	"log"
	"os"
	"time"

	uv "github.com/charmbracelet/ultraviolet"
	"github.com/lucasb-eyer/go-colorful"
)

func main() {
	if err := run(); err != nil {
		fmt.Fprintf(os.Stderr, "error: %v\n", err)
		os.Exit(1)
	}
}

func run() error {
	// Initialize terminal
	t := uv.DefaultTerminal()

	t.EnterAltScreen()
	if err := t.Start(); err != nil {
		return fmt.Errorf("failed to start: %w", err)
	}

	app := &App{
		term:         t,
		fpsWidget:    newFpsWidget(),
		colorsWidget: newColorsWidget(),
	}

	ctx := context.Background()

	if err := app.run(ctx); err != nil {
		return err
	}

	if err := t.Shutdown(ctx); err != nil {
		return fmt.Errorf("failed to shutdown: %w", err)
	}

	return nil
}

type App struct {
	term         *uv.Terminal
	fpsWidget    *FpsWidget
	colorsWidget *ColorsWidget
}

type FpsWidget struct {
	frameCount  int
	lastInstant time.Time
	fps         float32
}

type ColorsWidget struct {
	colors     [][]color.Color
	frameCount int
	width      int
	height     int
}

func newFpsWidget() *FpsWidget {
	return &FpsWidget{
		frameCount:  0,
		lastInstant: time.Now(),
		fps:         0,
	}
}

func newColorsWidget() *ColorsWidget {
	return &ColorsWidget{
		colors:     [][]color.Color{},
		frameCount: 0,
		width:      0,
		height:     0,
	}
}

func (a *App) run(ctx context.Context) error {
	// Ticker for rendering at ~60 FPS
	ticker := time.NewTicker(16 * time.Millisecond)
	defer ticker.Stop()

	// Main loop
	for {
		select {
		case ev := <-a.term.Events():
			switch ev := ev.(type) {
			case uv.WindowSizeEvent:
				a.term.Resize(ev.Width, ev.Height)
			case uv.KeyPressEvent:
				// Quit on any key press
				return nil
			}
		case <-ticker.C:
			if err := a.render(); err != nil {
				return err
			}
		}
	}
}

func (a *App) render() error {
	size := a.term.Size()
	width, height := size.Width, size.Height

	// Render title (centered in left portion, leaving 8 chars for FPS on right)
	title := "colors_rgb example. Press q to quit"
	titleAreaWidth := width - 8
	titleX := (titleAreaWidth / 2) - len(title)/2
	if titleX < 0 {
		titleX = 0
	}

	// Style for title (white text, default background)
	titleStyle := uv.Style{
		Fg: color.RGBA{R: 255, G: 255, B: 255, A: 255},
	}

	// Render title characters
	for i, ch := range title {
		cell := &uv.Cell{
			Content: string(ch),
			Style:   titleStyle,
			Width:   1,
		}
		a.term.SetCell(titleX+i, 0, cell)
	}

	// Render FPS on the right side
	a.fpsWidget.calculateFps()
	if a.fpsWidget.fps > 0 {
		fpsText := fmt.Sprintf("%.1f fps", a.fpsWidget.fps)
		fpsX := width - len(fpsText)
		if fpsX < 0 {
			fpsX = 0
		}

		for i, ch := range fpsText {
			cell := &uv.Cell{
				Content: string(ch),
				Style:   titleStyle,
				Width:   1,
			}
			a.term.SetCell(fpsX+i, 0, cell)
		}
	}

	// Render colors widget (starting from row 1, right after title)
	colorsHeight := height - 1
	if colorsHeight > 0 {
		a.colorsWidget.setupColors(width, colorsHeight)
		a.colorsWidget.render(a.term, 1, width)
	}

	if err := a.term.Display(); err != nil {
		log.Printf("display error: %v", err)
	}

	return nil
}

func (f *FpsWidget) calculateFps() {
	f.frameCount++
	elapsed := time.Since(f.lastInstant)
	if elapsed > time.Second && f.frameCount > 2 {
		f.fps = float32(f.frameCount) / float32(elapsed.Seconds())
		f.frameCount = 0
		f.lastInstant = time.Now()
	}
}

func (c *ColorsWidget) setupColors(width, height int) {
	// Double the height because each screen row has two rows of half block pixels
	pixelHeight := height * 2

	// Only update colors if size has changed
	if len(c.colors) == pixelHeight && c.width == width {
		return
	}

	c.width = width
	c.height = pixelHeight
	c.colors = make([][]color.Color, pixelHeight)

	for y := 0; y < pixelHeight; y++ {
		row := make([]color.Color, width)
		for x := 0; x < width; x++ {
			// Convert from HSV to RGB
			// Hue: 0-360 across width
			// Saturation: max (1.0)
			// Value: 0 at top to 1.0 at bottom
			hue := float64(x) * 360.0 / float64(width)
			value := float64(pixelHeight-y) / float64(pixelHeight)
			saturation := 1.0

			// Convert HSV to RGB using go-colorful
			col := colorful.Hsv(hue, saturation, value)
			r, g, b := col.RGB255()

			row[x] = color.RGBA{R: r, G: g, B: b, A: 255}
		}
		c.colors[y] = row
	}
}

func (c *ColorsWidget) render(term *uv.Terminal, startRow, width int) {
	screenHeight := c.height / 2 // screen rows (each contains 2 pixel rows)

	for y := 0; y < screenHeight; y++ {
		for x := 0; x < width; x++ {
			// Animate the colors by shifting the x index by the frame number
			xi := (x + c.frameCount) % width

			// Render a half block character for each row of pixels with the foreground color
			// set to the color of the top pixel and the background color set to the color of
			// the pixel below it
			fg := c.colors[y*2][xi]
			bg := c.colors[y*2+1][xi]

			style := uv.Style{
				Fg: fg,
				Bg: bg,
			}

			cell := &uv.Cell{
				Content: "▀",
				Style:   style,
				Width:   1,
			}

			term.SetCell(x, startRow+y, cell)
		}
	}

	c.frameCount++
}
