/// Demonstrates Unicode block mosaic rendering
///
/// This example shows how to use Zaz's mosaic module to render images
/// as Unicode block art with ANSI colors in the terminal.
use zaz::{MosaicConfig, SymbolSet, render_mosaic};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load the yellow.png image from resources
    let img = image::open("examples/resources/yellow.png")?;

    // Convert to RGB8 format
    let rgb_img = img.to_rgb8();
    let (width, height) = rgb_img.dimensions();
    let data = rgb_img.as_raw();

    println!("Mosaic Rendering Demo\n");
    println!("Image dimensions: {}x{} pixels\n", width, height);

    // Demo 1: Half blocks only (default)
    println!("1. Half Blocks (▀▄):");
    let config1 = MosaicConfig::with_width(60);
    let art1 = render_mosaic(data, width as usize, height as usize, &config1);
    println!("{}", art1);

    // Demo 2: All blocks with lower threshold for more detail
    println!("\n2. All Blocks with Lower Threshold:");
    let config2 = MosaicConfig::with_width(60)
        .threshold(100)
        .symbols(SymbolSet::All);
    let art2 = render_mosaic(data, width as usize, height as usize, &config2);
    println!("{}", art2);

    // Demo 3: Smaller width (30 cells)
    println!("\n3. Smaller Size (30 cells wide):");
    let config3 = MosaicConfig::with_width(30).symbols(SymbolSet::Quarter);
    let art3 = render_mosaic(data, width as usize, height as usize, &config3);
    println!("{}", art3);

    // Demo 4: Fixed dimensions with high threshold (more contrast)
    println!("\n4. Fixed Dimensions (40x20) with High Threshold:");
    let config4 = MosaicConfig::with_width(40)
        .height(20)
        .threshold(160)
        .symbols(SymbolSet::All);
    let art4 = render_mosaic(data, width as usize, height as usize, &config4);
    println!("{}", art4);

    Ok(())
}
