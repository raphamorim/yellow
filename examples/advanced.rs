use zaz::{ACS_BULLET, ACS_DIAMOND, Color, Panel, Screen};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut scr = Screen::init()?;

    // Demonstrate ACS characters with box drawing
    scr.draw_box()?;
    scr.mvprint(1, 2, "Advanced Features Demo")?;

    // Panel demonstration
    scr.mvprint(3, 2, "Creating overlapping panels...")?;

    let mut win1 = scr.newwin(8, 30, 5, 5)?;
    win1.draw_box()?;
    win1.set_fg(Color::Green)?;
    win1.mvprint(2, 2, "Panel 1")?;
    win1.mvprint(3, 2, &format!("{} ACS Diamond", ACS_DIAMOND.as_char()))?;

    let mut win2 = scr.newwin(8, 30, 8, 15)?;
    win2.draw_box()?;
    win2.set_fg(Color::Cyan)?;
    win2.mvprint(2, 2, "Panel 2")?;
    win2.mvprint(3, 2, &format!("{} ACS Bullet", ACS_BULLET.as_char()))?;

    // Create panels
    let mut panel1 = Panel::new(win1)?;
    let mut panel2 = Panel::new(win2)?;

    // Use wnoutrefresh and doupdate for efficient rendering
    panel1.wnoutrefresh()?;
    panel2.wnoutrefresh()?;
    Screen::doupdate()?;

    scr.mvprint(18, 2, "Press any key within 3 seconds...")?;
    scr.refresh()?;

    // Demonstrate getch_timeout
    match scr.getch_timeout(3000)? {
        Some(key) => {
            scr.mvprint(19, 2, &format!("You pressed: {:?}", key))?;
        }
        None => {
            scr.mvprint(19, 2, "Timeout! No key pressed.")?;
        }
    }

    scr.mvprint(21, 2, "Press any key to exit...")?;
    scr.refresh()?;
    scr.getch()?;

    scr.endwin()?;
    println!("Advanced demo complete!");

    Ok(())
}
