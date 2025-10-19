/// Debug tool to output raw escape sequences
/// This helps verify the format is correct
use zaz::{ImageFormat, KittyImage, SixelImage};

fn main() {
    // Test 1: Minimal Kitty image (2x2 red square)
    println!("1. Kitty Protocol - 2x2 red square (RGB format):");
    let kitty_data = vec![
        255, 0, 0, 255, 0, 0, // Row 1: red, red
        255, 0, 0, 255, 0, 0, // Row 2: red, red
    ];

    let kitty = KittyImage::new(&kitty_data, ImageFormat::Rgb).with_pixel_size(2, 2);
    match kitty.to_sequence() {
        Ok(seq) => {
            println!("Sequence length: {} bytes", seq.len());
            println!(
                "First 100 chars: {:?}",
                &seq.chars().take(100).collect::<String>()
            );
            println!(
                "Last 50 chars: {:?}",
                &seq.chars()
                    .rev()
                    .take(50)
                    .collect::<String>()
                    .chars()
                    .rev()
                    .collect::<String>()
            );
            println!("\nFull sequence (escaped):");
            for ch in seq.chars() {
                if ch == '\x1b' {
                    print!("ESC");
                } else if ch.is_ascii_control() {
                    print!("<{}>", ch as u8);
                } else {
                    print!("{}", ch);
                }
            }
            println!("\n");
        }
        Err(e) => println!("Error: {:?}", e),
    }

    // Test 2: Minimal Sixel image (4x4 white square)
    println!("\n2. Sixel Protocol - 4x4 white square:");
    let sixel_data = vec![255u8; 48]; // 4x4 pixels * 3 bytes
    let sixel = SixelImage::from_rgb(&sixel_data, 4, 4);
    match sixel.to_sequence() {
        Ok(seq) => {
            println!("Sequence length: {} bytes", seq.len());
            println!(
                "First 150 chars: {:?}",
                &seq.chars().take(150).collect::<String>()
            );
            println!("\nFull sequence (escaped):");
            for ch in seq.chars() {
                if ch == '\x1b' {
                    print!("ESC");
                } else if ch.is_ascii_control() {
                    print!("<{}>", ch as u8);
                } else {
                    print!("{}", ch);
                }
            }
            println!("\n");
        }
        Err(e) => println!("Error: {:?}", e),
    }

    println!("\n=== Expected Formats ===");
    println!("Kitty should be: ESC_Ga=T,f=24,t=d;<base64>ESC\\");
    println!("Sixel should be: ESCP0;0;0q\"1;1;w;h#<colors><data>ESC\\");
}
