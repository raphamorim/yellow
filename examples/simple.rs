use zaz::{Attr, Color, Screen};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize screen
    let mut scr = Screen::init()?;

    // Simple hello world
    scr.mvprint(5, 10, "Hello, World!")?;

    // Add some color
    scr.set_fg(Color::Red)?;
    scr.attron(Attr::BOLD)?;
    scr.mvprint(7, 10, "Bold Red Text")?;

    scr.refresh()?;

    // Wait for keypress
    scr.getch()?;

    // Clean up
    scr.endwin()?;

    Ok(())
}
