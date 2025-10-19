/// Unicode block-based image rendering
///
/// Converts raw pixel data into terminal-displayable Unicode art using
/// block characters (▀▄█ etc.) with ANSI color codes.
use std::fmt::Write;

/// Unicode block character with coverage information
#[derive(Debug, Clone, Copy)]
struct Block {
    ch: char,
    /// Which quadrants are filled: [upper-left, upper-right, lower-left, lower-right]
    coverage: [bool; 4],
}

/// Available block symbols
const HALF_BLOCKS: &[Block] = &[
    Block {
        ch: '▀',
        coverage: [true, true, false, false],
    }, // Upper half
    Block {
        ch: '▄',
        coverage: [false, false, true, true],
    }, // Lower half
    Block {
        ch: ' ',
        coverage: [false, false, false, false],
    }, // Empty
    Block {
        ch: '█',
        coverage: [true, true, true, true],
    }, // Full
];

const QUARTER_BLOCKS: &[Block] = &[
    Block {
        ch: '▘',
        coverage: [true, false, false, false],
    }, // Upper left
    Block {
        ch: '▝',
        coverage: [false, true, false, false],
    }, // Upper right
    Block {
        ch: '▖',
        coverage: [false, false, true, false],
    }, // Lower left
    Block {
        ch: '▗',
        coverage: [false, false, false, true],
    }, // Lower right
    Block {
        ch: '▌',
        coverage: [true, false, true, false],
    }, // Left half
    Block {
        ch: '▐',
        coverage: [false, true, false, true],
    }, // Right half
];

const COMPLEX_BLOCKS: &[Block] = &[
    Block {
        ch: '▙',
        coverage: [true, false, true, true],
    }, // UL + lower half
    Block {
        ch: '▟',
        coverage: [false, true, true, true],
    }, // UR + lower half
    Block {
        ch: '▛',
        coverage: [true, true, true, false],
    }, // Upper half + LL
    Block {
        ch: '▜',
        coverage: [true, true, false, true],
    }, // Upper half + LR
    Block {
        ch: '▚',
        coverage: [true, false, false, true],
    }, // UL + LR diagonal
    Block {
        ch: '▞',
        coverage: [false, true, true, false],
    }, // UR + LL diagonal
];

/// Symbol set to use for rendering
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SymbolSet {
    /// Only half blocks (▀▄)
    Half,
    /// Half and quarter blocks (▀▄▌▐ etc.)
    Quarter,
    /// All available blocks including complex patterns
    All,
}

/// Configuration for mosaic rendering
#[derive(Debug, Clone)]
pub struct MosaicConfig {
    /// Output width in terminal cells (0 = use image width)
    pub width: usize,
    /// Output height in terminal cells (0 = auto-calculate from aspect ratio)
    pub height: usize,
    /// Luminance threshold for considering a pixel "set" (0-255)
    pub threshold: u8,
    /// Which symbol set to use
    pub symbols: SymbolSet,
}

impl Default for MosaicConfig {
    fn default() -> Self {
        Self {
            width: 0,
            height: 0,
            threshold: 128,
            symbols: SymbolSet::Half,
        }
    }
}

impl MosaicConfig {
    /// Create a new config with specified width
    pub fn with_width(width: usize) -> Self {
        Self {
            width,
            ..Default::default()
        }
    }

    /// Set output height
    pub fn height(mut self, height: usize) -> Self {
        self.height = height;
        self
    }

    /// Set luminance threshold
    pub fn threshold(mut self, threshold: u8) -> Self {
        self.threshold = threshold;
        self
    }

    /// Set symbol set
    pub fn symbols(mut self, symbols: SymbolSet) -> Self {
        self.symbols = symbols;
        self
    }
}

/// RGB color
#[derive(Debug, Clone, Copy)]
struct Rgb {
    r: u8,
    g: u8,
    b: u8,
}

impl Rgb {
    fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    /// Calculate luminance (perceived brightness)
    fn luminance(&self) -> u8 {
        // Weighted RGB for human perception
        // Source: https://www.w3.org/TR/AERT/#color-contrast
        (self.r as f32 * 0.299 + self.g as f32 * 0.587 + self.b as f32 * 0.114) as u8
    }

    /// Convert to ANSI escape code for foreground color
    fn to_ansi_fg(&self) -> String {
        format!("\x1b[38;2;{};{};{}m", self.r, self.g, self.b)
    }

    /// Convert to ANSI escape code for background color
    fn to_ansi_bg(&self) -> String {
        format!("\x1b[48;2;{};{};{}m", self.r, self.g, self.b)
    }
}

/// Average multiple RGB colors
fn average_colors(colors: &[Rgb]) -> Rgb {
    if colors.is_empty() {
        return Rgb::new(0, 0, 0);
    }

    let mut sum_r = 0u32;
    let mut sum_g = 0u32;
    let mut sum_b = 0u32;

    for c in colors {
        sum_r += c.r as u32;
        sum_g += c.g as u32;
        sum_b += c.b as u32;
    }

    let count = colors.len() as u32;
    Rgb::new(
        (sum_r / count) as u8,
        (sum_g / count) as u8,
        (sum_b / count) as u8,
    )
}

/// Render RGB image data as Unicode block art
///
/// # Arguments
/// * `data` - Raw RGB pixel data (3 bytes per pixel, row-major order)
/// * `width` - Image width in pixels
/// * `height` - Image height in pixels
/// * `config` - Rendering configuration
///
/// # Returns
/// String containing Unicode art with ANSI color codes
///
/// # Example
/// ```
/// use zaz::{render_mosaic, MosaicConfig};
///
/// // Create a 4x4 red square
/// let data = vec![255u8, 0, 0].repeat(16);
/// let art = render_mosaic(&data, 4, 4, &MosaicConfig::with_width(2));
/// println!("{}", art);
/// ```
pub fn render_mosaic(data: &[u8], width: usize, height: usize, config: &MosaicConfig) -> String {
    // Calculate output dimensions
    let out_width = if config.width > 0 {
        config.width
    } else {
        width
    };

    let out_height = if config.height > 0 {
        config.height
    } else {
        // Auto-calculate height maintaining aspect ratio
        // Terminal chars are ~2x taller than wide
        ((out_width as f32 * height as f32 / width as f32) / 2.0).max(1.0) as usize
    };

    // Resize image if needed
    let resized = if width != out_width * 2 || height != out_height * 2 {
        resize_image(data, width, height, out_width * 2, out_height * 2)
    } else {
        data.to_vec()
    };

    let resized_width = out_width * 2;

    // Select block set
    let mut blocks = HALF_BLOCKS.to_vec();
    if config.symbols == SymbolSet::Quarter || config.symbols == SymbolSet::All {
        blocks.extend_from_slice(QUARTER_BLOCKS);
    }
    if config.symbols == SymbolSet::All {
        blocks.extend_from_slice(COMPLEX_BLOCKS);
    }

    let mut output = String::new();

    // Process image in 2x2 blocks (each becomes one terminal cell)
    for block_y in 0..out_height {
        for block_x in 0..out_width {
            // Extract 2x2 pixel block
            let px_y = block_y * 2;
            let px_x = block_x * 2;

            let mut pixels = [[Rgb::new(0, 0, 0); 2]; 2];
            for dy in 0..2 {
                for dx in 0..2 {
                    let y = px_y + dy;
                    let x = px_x + dx;
                    if y < out_height * 2 && x < resized_width {
                        let offset = (y * resized_width + x) * 3;
                        if offset + 2 < resized.len() {
                            pixels[dy][dx] =
                                Rgb::new(resized[offset], resized[offset + 1], resized[offset + 2]);
                        }
                    }
                }
            }

            // Determine which pixels are "set" based on threshold
            let mask = [
                [
                    pixels[0][0].luminance() >= config.threshold,
                    pixels[0][1].luminance() >= config.threshold,
                ],
                [
                    pixels[1][0].luminance() >= config.threshold,
                    pixels[1][1].luminance() >= config.threshold,
                ],
            ];

            // Find best matching block
            let pixel_mask_flat = [mask[0][0], mask[0][1], mask[1][0], mask[1][1]];
            let best_block = find_best_block(&pixel_mask_flat, &blocks);

            // Determine foreground and background colors
            let mut fg_pixels = Vec::new();
            let mut bg_pixels = Vec::new();

            for i in 0..4 {
                let y = i / 2;
                let x = i % 2;
                if best_block.coverage[i] {
                    fg_pixels.push(pixels[y][x]);
                } else {
                    bg_pixels.push(pixels[y][x]);
                }
            }

            let fg_color = average_colors(&fg_pixels);
            let bg_color = average_colors(&bg_pixels);

            // Write cell with colors
            write!(
                output,
                "{}{}{}",
                fg_color.to_ansi_fg(),
                bg_color.to_ansi_bg(),
                best_block.ch
            )
            .unwrap();
        }

        // Reset colors and newline
        output.push_str("\x1b[0m\n");
    }

    output
}

/// Find the block character that best matches the pixel mask
fn find_best_block(mask: &[bool; 4], blocks: &[Block]) -> Block {
    let mut best = blocks[0];
    let mut best_score = 4;

    for block in blocks {
        let mut score = 0;
        for i in 0..4 {
            if block.coverage[i] != mask[i] {
                score += 1;
            }
        }

        if score < best_score {
            best_score = score;
            best = *block;
        }

        if score == 0 {
            break; // Perfect match
        }
    }

    best
}

/// Simple nearest-neighbor image resizing
fn resize_image(data: &[u8], src_w: usize, src_h: usize, dst_w: usize, dst_h: usize) -> Vec<u8> {
    let mut result = vec![0u8; dst_w * dst_h * 3];

    for dst_y in 0..dst_h {
        for dst_x in 0..dst_w {
            let src_x = (dst_x * src_w) / dst_w;
            let src_y = (dst_y * src_h) / dst_h;

            let src_offset = (src_y * src_w + src_x) * 3;
            let dst_offset = (dst_y * dst_w + dst_x) * 3;

            if src_offset + 2 < data.len() && dst_offset + 2 < result.len() {
                result[dst_offset] = data[src_offset];
                result[dst_offset + 1] = data[src_offset + 1];
                result[dst_offset + 2] = data[src_offset + 2];
            }
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rgb_luminance() {
        let white = Rgb::new(255, 255, 255);
        assert_eq!(white.luminance(), 255);

        let black = Rgb::new(0, 0, 0);
        assert_eq!(black.luminance(), 0);

        let red = Rgb::new(255, 0, 0);
        assert!(red.luminance() > 0 && red.luminance() < 255);
    }

    #[test]
    fn test_average_colors() {
        let colors = vec![Rgb::new(0, 0, 0), Rgb::new(255, 255, 255)];
        let avg = average_colors(&colors);
        assert_eq!(avg.r, 127);
        assert_eq!(avg.g, 127);
        assert_eq!(avg.b, 127);
    }

    #[test]
    fn test_render_simple() {
        // 2x2 red square
        let data = vec![255u8, 0, 0, 255, 0, 0, 255, 0, 0, 255, 0, 0];
        let art = render_mosaic(&data, 2, 2, &MosaicConfig::with_width(1));
        assert!(!art.is_empty());
        assert!(art.contains('\x1b')); // Contains ANSI codes
    }

    #[test]
    fn test_config_builder() {
        let config = MosaicConfig::with_width(50)
            .height(25)
            .threshold(100)
            .symbols(SymbolSet::All);

        assert_eq!(config.width, 50);
        assert_eq!(config.height, 25);
        assert_eq!(config.threshold, 100);
        assert_eq!(config.symbols, SymbolSet::All);
    }

    #[test]
    fn test_block_matching() {
        // All pixels set -> should match full block
        let mask = [true, true, true, true];
        let best = find_best_block(&mask, HALF_BLOCKS);
        assert_eq!(best.ch, '█');

        // No pixels set -> should match empty
        let mask = [false, false, false, false];
        let best = find_best_block(&mask, HALF_BLOCKS);
        assert_eq!(best.ch, ' ');

        // Upper half set -> should match upper half block
        let mask = [true, true, false, false];
        let best = find_best_block(&mask, HALF_BLOCKS);
        assert_eq!(best.ch, '▀');
    }

    #[test]
    fn test_resize_image() {
        // 2x2 image -> 4x4
        let data = vec![255, 0, 0, 0, 255, 0, 0, 0, 255, 255, 255, 255];
        let resized = resize_image(&data, 2, 2, 4, 4);
        assert_eq!(resized.len(), 4 * 4 * 3);
    }
}
