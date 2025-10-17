use yellow::{Attr, Color, Screen};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut scr = Screen::init()?;

    // Draw border around screen
    scr.border('|', '|', '-', '-', '+', '+', '+', '+')?;

    // Title with colors
    scr.init_pair(1, Color::Yellow, Color::Blue)?;
    scr.color_pair(1)?;
    scr.attron(Attr::BOLD)?;
    scr.mvprint(2, 10, "Yellow Terminal Library Demo")?;
    scr.attroff(Attr::BOLD)?;

    // Reset colors
    scr.init_pair(2, Color::White, Color::Black)?;
    scr.color_pair(2)?;

    // Display some styled text
    scr.mvprint(4, 5, "Text Attributes:")?;

    scr.attron(Attr::BOLD)?;
    scr.mvprint(5, 7, "Bold text")?;
    scr.attroff(Attr::BOLD)?;

    scr.attron(Attr::ITALIC)?;
    scr.mvprint(6, 7, "Italic text")?;
    scr.attroff(Attr::ITALIC)?;

    scr.attron(Attr::UNDERLINE)?;
    scr.mvprint(7, 7, "Underlined text")?;
    scr.attroff(Attr::UNDERLINE)?;

    scr.attron(Attr::BOLD | Attr::UNDERLINE)?;
    scr.mvprint(8, 7, "Bold and underlined")?;
    scr.attrset(Attr::NORMAL)?;

    // Color examples
    scr.mvprint(10, 5, "Colors:")?;

    scr.set_fg(Color::Red)?;
    scr.mvprint(11, 7, "Red")?;

    scr.set_fg(Color::Green)?;
    scr.mvprint(11, 15, "Green")?;

    scr.set_fg(Color::Blue)?;
    scr.mvprint(11, 25, "Blue")?;

    scr.set_fg(Color::Magenta)?;
    scr.mvprint(11, 35, "Magenta")?;

    // RGB color
    scr.set_fg(Color::Rgb(255, 128, 0))?;
    scr.mvprint(12, 7, "RGB Orange (255, 128, 0)")?;

    // Reset to white
    scr.set_fg(Color::White)?;

    // Create a window
    scr.mvprint(14, 5, "Window example:")?;
    let mut win = scr.newwin(6, 30, 15, 10)?;
    win.border('|', '|', '-', '-', '+', '+', '+', '+')?;
    win.set_fg(Color::Cyan)?;
    win.mvprint(2, 2, "This is inside a window")?;
    win.set_fg(Color::Yellow)?;
    win.mvprint(3, 2, "at position (15, 10)")?;
    win.refresh()?;

    // Instructions
    scr.mvprint(22, 5, "Press any key to exit...")?;

    scr.refresh()?;

    // Wait for key press
    let key = scr.getch()?;

    // Show what key was pressed (briefly)
    scr.mvprint(23, 5, &format!("You pressed: {:?}", key))?;
    scr.refresh()?;

    std::thread::sleep(std::time::Duration::from_millis(500));

    scr.endwin()?;

    println!("Demo complete!");

    Ok(())
}
