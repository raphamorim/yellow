//! A Yellow example that shows the full range of RGB colors that can be displayed in the terminal.
//!
//! Requires a terminal that supports 24-bit color (true color) and unicode.
//!
//! This example demonstrates:
//! - RGB color rendering with Yellow
//! - Double-buffering for smooth animation
//! - FPS calculation and display
//! - Using half-block characters for higher resolution color display
//!
//! Press any key to quit.

use std::time::{Duration, Instant};

use palette::convert::FromColorUnclamped;
use palette::{Okhsv, Srgb};
use zaz::{Attr, Color, Screen};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let app = App::new()?;
    app.run()?;
    Ok(())
}

struct App {
    screen: Screen,
    fps_widget: FpsWidget,
    colors_widget: ColorsWidget,
}

/// A widget that displays the current frames per second
struct FpsWidget {
    /// The number of elapsed frames that have passed - used to calculate the fps
    frame_count: usize,
    /// The last instant that the fps was calculated
    last_instant: Instant,
    /// The current frames per second
    fps: Option<f32>,
}

/// A widget that displays the full range of RGB colors that can be displayed in the terminal.
///
/// This widget is animated and will change colors over time.
struct ColorsWidget {
    /// The colors to render - should be double the height of the area as we render two rows of
    /// pixels for each row of the widget using the half block character.
    colors: Vec<Vec<Color>>,
    /// the number of elapsed frames that have passed - used to animate the colors by shifting the
    /// x index by the frame number
    frame_count: usize,
    /// cached dimensions
    width: usize,
    height: usize,
}

impl App {
    fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let screen = Screen::init()?;
        Ok(Self {
            screen,
            fps_widget: FpsWidget::new(),
            colors_widget: ColorsWidget::new(),
        })
    }

    fn run(mut self) -> Result<(), Box<dyn std::error::Error>> {
        let mut running = true;

        while running {
            self.render()?;
            running = self.handle_events()?;
        }

        self.screen.endwin()?;
        Ok(())
    }

    fn render(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let (rows, cols) = self.screen.get_size()?;

        // Clear screen
        self.screen.clear()?;

        // Render top bar with title and FPS
        // Draw a separator line
        let separator = "─".repeat(cols as usize);
        self.screen.mvprint(1, 0, &separator)?;

        // Render title (centered)
        let title = "colors_rgb example. Press any key to quit";
        let title_x = (cols as usize / 2).saturating_sub(title.len() / 2);
        self.screen.mvprint(0, title_x as u16, title)?;

        // Render FPS in top left with styling
        self.fps_widget.calculate_fps();
        if let Some(fps) = self.fps_widget.fps {
            self.screen.attron(Attr::BOLD)?;
            self.screen.set_fg(Color::Rgb(100, 200, 255))?;
            let fps_text = format!("FPS: {:.1}", fps);
            self.screen.mvprint(0, 0, &fps_text)?;
            self.screen.attroff(Attr::BOLD)?;
            self.screen.set_fg(Color::Rgb(255, 255, 255))?;
        }

        // Render colors widget (starting from row 2, after separator)
        let colors_height = rows.saturating_sub(2);
        self.colors_widget.setup_colors(cols, colors_height);
        self.colors_widget.render(&mut self.screen, 2, cols)?;

        self.screen.refresh()?;
        Ok(())
    }

    fn handle_events(&mut self) -> Result<bool, Box<dyn std::error::Error>> {
        // Check for input with timeout to target ~60 FPS (16ms per frame)
        if let Some(_key) = self.screen.getch_timeout(16)? {
            return Ok(false); // Any key press quits
        }

        Ok(true) // Keep running
    }
}

impl FpsWidget {
    fn new() -> Self {
        Self {
            frame_count: 0,
            last_instant: Instant::now(),
            fps: None,
        }
    }

    /// Update the fps calculation.
    ///
    /// This updates the fps once a second, but only if the widget has rendered at least 2 frames
    /// since the last calculation.
    fn calculate_fps(&mut self) {
        self.frame_count += 1;
        let elapsed = self.last_instant.elapsed();
        if elapsed > Duration::from_secs(1) && self.frame_count > 2 {
            self.fps = Some(self.frame_count as f32 / elapsed.as_secs_f32());
            self.frame_count = 0;
            self.last_instant = Instant::now();
        }
    }
}

impl ColorsWidget {
    fn new() -> Self {
        Self {
            colors: Vec::new(),
            frame_count: 0,
            width: 0,
            height: 0,
        }
    }

    /// Setup the colors to render.
    ///
    /// This is called once per frame to setup the colors to render. It caches the colors so that
    /// they don't need to be recalculated every frame.
    fn setup_colors(&mut self, width: u16, height: u16) {
        let width = width as usize;
        // double the height because each screen row has two rows of half block pixels
        let height = (height as usize) * 2;

        // only update the colors if the size has changed since the last time we rendered
        if self.colors.len() == height && self.width == width {
            return;
        }

        self.width = width;
        self.height = height;
        self.colors = Vec::with_capacity(height);

        for y in 0..height {
            let mut row = Vec::with_capacity(width);
            for x in 0..width {
                let hue = x as f32 * 360.0 / width as f32;
                let value = (height - y) as f32 / height as f32;
                let saturation = Okhsv::max_saturation();
                let color = Okhsv::new(hue, saturation, value);
                let color = Srgb::<f32>::from_color_unclamped(color);
                let color: Srgb<u8> = color.into_format();
                row.push(Color::Rgb(color.red, color.green, color.blue));
            }
            self.colors.push(row);
        }
    }

    /// Render the colors widget
    fn render(
        &mut self,
        screen: &mut Screen,
        start_row: u16,
        width: u16,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let width = width as usize;
        let height = self.height / 2; // screen rows (each contains 2 pixel rows)

        for y in 0..height {
            for x in 0..width {
                // animate the colors by shifting the x index by the frame number
                let xi = (x + self.frame_count) % width;

                // render a half block character for each row of pixels with the foreground color
                // set to the color of the top pixel and the background color set to the color of
                // the pixel below it
                let fg = self.colors[y * 2][xi];
                let bg = self.colors[y * 2 + 1][xi];

                screen.set_fg(fg)?;
                screen.set_bg(bg)?;
                screen.mvaddch(start_row + y as u16, x as u16, '▀')?;
            }
        }

        self.frame_count += 1;
        Ok(())
    }
}
