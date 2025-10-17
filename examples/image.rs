use yellow::{ImageFormat, ImagePlacement, KittyImage, Screen, SixelImage};

/// Generate a simple gradient image in RGB format
fn generate_gradient(width: u32, height: u32) -> Vec<u8> {
    let mut data = Vec::with_capacity((width * height * 3) as usize);

    for y in 0..height {
        for x in 0..width {
            let r = ((x as f32 / width as f32) * 255.0) as u8;
            let g = ((y as f32 / height as f32) * 255.0) as u8;
            let b = 128;

            data.push(r);
            data.push(g);
            data.push(b);
        }
    }

    data
}

/// Generate a checkerboard pattern in RGB format
fn generate_checkerboard(width: u32, height: u32, cell_size: u32) -> Vec<u8> {
    let mut data = Vec::with_capacity((width * height * 3) as usize);

    for y in 0..height {
        for x in 0..width {
            let is_white = ((x / cell_size) + (y / cell_size)) % 2 == 0;
            let value = if is_white { 255 } else { 0 };

            data.push(value);
            data.push(value);
            data.push(value);
        }
    }

    data
}

/// Generate a color palette image
fn generate_palette(width: u32, height: u32) -> Vec<u8> {
    let mut data = Vec::with_capacity((width * height * 3) as usize);
    let colors_per_row = 8;
    let color_width = width / colors_per_row;

    let colors = [
        (255, 0, 0),     // Red
        (0, 255, 0),     // Green
        (0, 0, 255),     // Blue
        (255, 255, 0),   // Yellow
        (255, 0, 255),   // Magenta
        (0, 255, 255),   // Cyan
        (255, 255, 255), // White
        (128, 128, 128), // Gray
    ];

    for _y in 0..height {
        for x in 0..width {
            let color_idx = (x / color_width) as usize % colors.len();
            let (r, g, b) = colors[color_idx];

            data.push(r);
            data.push(g);
            data.push(b);
        }
    }

    data
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut scr = Screen::init()?;

    scr.clear()?;
    scr.draw_box()?;
    scr.mvprint(1, 2, "Yellow Image Protocol Demo")?;
    scr.mvprint(3, 2, "This demo shows both Kitty and Sixel image protocols")?;

    // Ask which protocol to use
    scr.mvprint(5, 2, "Choose protocol:")?;
    scr.mvprint(6, 4, "1 - Kitty Graphics Protocol")?;
    scr.mvprint(7, 4, "2 - Sixel Graphics Protocol")?;
    scr.mvprint(8, 4, "q - Quit")?;
    scr.mvprint(10, 2, "Enter choice: ")?;
    scr.refresh()?;

    let choice = scr.getch()?;

    match choice {
        yellow::Key::Char('1') => demo_kitty(&mut scr)?,
        yellow::Key::Char('2') => demo_sixel(&mut scr)?,
        _ => {}
    }

    scr.endwin()?;
    println!("Image demo complete!");

    Ok(())
}

fn demo_kitty(scr: &mut Screen) -> Result<(), Box<dyn std::error::Error>> {
    scr.clear()?;
    scr.mvprint(1, 2, "Kitty Graphics Protocol Demo")?;
    scr.mvprint(3, 2, "Displaying images using Kitty protocol...")?;
    scr.refresh()?;

    // Demo 1: Simple gradient
    scr.mvprint(5, 2, "1. Gradient (20x10):")?;
    scr.refresh()?;

    let gradient_data = generate_gradient(20, 10);
    let gradient = KittyImage::new(&gradient_data, ImageFormat::Rgb)
        .with_pixel_size(20, 10)
        .with_image_id(1)
        .placement(ImagePlacement::at(7, 4).with_size(10, 5));

    scr.display_kitty_image(&gradient)?;
    scr.refresh()?;

    std::thread::sleep(std::time::Duration::from_secs(2));

    // Demo 2: Checkerboard
    scr.mvprint(13, 2, "2. Checkerboard (40x20):")?;
    scr.refresh()?;

    let check_data = generate_checkerboard(40, 20, 4);
    let checkerboard = KittyImage::new(&check_data, ImageFormat::Rgb)
        .with_pixel_size(40, 20)
        .with_image_id(2)
        .placement(ImagePlacement::at(15, 4).with_size(15, 8));

    scr.display_kitty_image(&checkerboard)?;
    scr.refresh()?;

    std::thread::sleep(std::time::Duration::from_secs(2));

    // Demo 3: Color palette
    scr.mvprint(24, 2, "3. Color Palette (64x8):")?;
    scr.refresh()?;

    let palette_data = generate_palette(64, 8);
    let palette = KittyImage::new(&palette_data, ImageFormat::Rgb)
        .with_pixel_size(64, 8)
        .with_image_id(3)
        .placement(ImagePlacement::at(26, 4).with_size(20, 3));

    scr.display_kitty_image(&palette)?;
    scr.refresh()?;

    scr.mvprint(30, 2, "Press any key to clean up and exit...")?;
    scr.refresh()?;
    scr.getch()?;

    // Clean up images
    scr.delete_all_kitty_images()?;
    scr.refresh()?;

    Ok(())
}

fn demo_sixel(scr: &mut Screen) -> Result<(), Box<dyn std::error::Error>> {
    scr.clear()?;
    scr.mvprint(1, 2, "Sixel Graphics Protocol Demo")?;
    scr.mvprint(3, 2, "Displaying images using Sixel protocol...")?;
    scr.mvprint(5, 2, "Note: Sixel support varies by terminal")?;
    scr.refresh()?;

    // Demo: Simple gradient using Sixel
    scr.mvprint(7, 2, "Gradient (32x16):")?;
    scr.move_cursor(9, 2)?;
    scr.refresh()?;

    let gradient_data = generate_gradient(32, 16);
    let sixel = SixelImage::from_rgb(&gradient_data, 32, 16);

    scr.display_sixel_image(&sixel)?;
    scr.refresh()?;

    scr.mvprint(20, 2, "Press any key to continue...")?;
    scr.refresh()?;
    scr.getch()?;

    // Demo: Checkerboard
    scr.clear()?;
    scr.mvprint(1, 2, "Checkerboard Pattern (40x20):")?;
    scr.move_cursor(3, 2)?;
    scr.refresh()?;

    let check_data = generate_checkerboard(40, 20, 5);
    let sixel_check = SixelImage::from_rgb(&check_data, 40, 20);

    scr.display_sixel_image(&sixel_check)?;
    scr.refresh()?;

    scr.mvprint(20, 2, "Press any key to continue...")?;
    scr.refresh()?;
    scr.getch()?;

    // Demo: Color palette
    scr.clear()?;
    scr.mvprint(1, 2, "Color Palette (64x16):")?;
    scr.move_cursor(3, 2)?;
    scr.refresh()?;

    let palette_data = generate_palette(64, 16);
    let sixel_palette = SixelImage::from_rgb(&palette_data, 64, 16);

    scr.display_sixel_image(&sixel_palette)?;
    scr.refresh()?;

    scr.mvprint(20, 2, "Press any key to exit...")?;
    scr.refresh()?;
    scr.getch()?;

    Ok(())
}
