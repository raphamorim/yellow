use zaz::{Color, Key, KittyFlags, Screen};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut scr = Screen::init()?;

    scr.clear()?;
    scr.draw_box()?;
    scr.mvprint(1, 2, "Kitty Keyboard Protocol Demo")?;
    scr.mvprint(3, 2, "This demo enables enhanced keyboard reporting")?;
    scr.mvprint(4, 2, "Try pressing keys with modifiers (Ctrl, Alt, Shift)")?;
    scr.mvprint(6, 2, "Press 'q' to quit")?;

    // Enable Kitty keyboard protocol with disambiguation and event types
    scr.push_kitty_keyboard(KittyFlags::DISAMBIGUATE | KittyFlags::EVENT_TYPES)?;
    scr.refresh()?;

    let mut row = 8;
    let max_row = 20;

    loop {
        let key = scr.getch()?;

        // Clear previous message
        scr.mvprint(row, 2, &" ".repeat(70))?;

        match key {
            Key::Char('q') => break,
            Key::Enhanced(event) => {
                let mut msg = format!("Enhanced: code={}", event.code);

                if let Some(ch) = char::from_u32(event.code) {
                    msg.push_str(&format!(" ('{}')", ch));
                }

                if event.is_shift() {
                    msg.push_str(" +Shift");
                }
                if event.is_ctrl() {
                    msg.push_str(" +Ctrl");
                }
                if event.is_alt() {
                    msg.push_str(" +Alt");
                }
                if event.is_super() {
                    msg.push_str(" +Super");
                }

                msg.push_str(&format!(
                    " [{}]",
                    match event.event_type {
                        zaz::KeyEventType::Press => "Press",
                        zaz::KeyEventType::Repeat => "Repeat",
                        zaz::KeyEventType::Release => "Release",
                    }
                ));

                if let Some(shifted) = event.shifted_key {
                    if let Some(ch) = char::from_u32(shifted) {
                        msg.push_str(&format!(" shifted='{}'", ch));
                    }
                }

                scr.set_fg(Color::Green)?;
                scr.mvprint(row, 2, &msg)?;
                scr.set_fg(Color::White)?;
            }
            Key::Char(ch) => {
                scr.set_fg(Color::Yellow)?;
                scr.mvprint(row, 2, &format!("Char: '{}'", ch))?;
                scr.set_fg(Color::White)?;
            }
            Key::Ctrl(ch) => {
                scr.set_fg(Color::Cyan)?;
                scr.mvprint(row, 2, &format!("Ctrl+{}", ch))?;
                scr.set_fg(Color::White)?;
            }
            Key::Alt(ch) => {
                scr.set_fg(Color::Magenta)?;
                scr.mvprint(row, 2, &format!("Alt+{}", ch))?;
                scr.set_fg(Color::White)?;
            }
            Key::Up => scr.mvprint(row, 2, "Arrow: Up")?,
            Key::Down => scr.mvprint(row, 2, "Arrow: Down")?,
            Key::Left => scr.mvprint(row, 2, "Arrow: Left")?,
            Key::Right => scr.mvprint(row, 2, "Arrow: Right")?,
            Key::F(n) => {
                scr.set_fg(Color::Blue)?;
                scr.mvprint(row, 2, &format!("Function key: F{}", n))?;
                scr.set_fg(Color::White)?;
            }
            _ => {
                scr.mvprint(row, 2, &format!("Other: {:?}", key))?;
            }
        }

        scr.refresh()?;

        row += 1;
        if row >= max_row {
            row = 8;
            // Clear the event area
            for r in 8..max_row {
                scr.mvprint(r, 2, &" ".repeat(70))?;
            }
        }
    }

    // Restore keyboard mode
    scr.pop_kitty_keyboard()?;
    scr.refresh()?;

    scr.endwin()?;
    println!("Kitty keyboard protocol demo complete!");

    Ok(())
}
