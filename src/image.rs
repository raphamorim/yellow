/// Terminal image protocol support
///
/// Supports both Sixel and Kitty image protocols for displaying images in the terminal.
///
/// # Sixel
/// Legacy protocol from DEC terminals, widely supported
///
/// # Kitty
/// Modern protocol with better performance and features
use std::fmt::Write;

/// Image transmission format
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageFormat {
    /// PNG format
    Png,
    /// JPEG format
    Jpeg,
    /// GIF format
    Gif,
    /// RGB raw data
    Rgb,
    /// RGBA raw data
    Rgba,
}

/// Image protocol to use
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageProtocol {
    /// Sixel graphics protocol
    Sixel,
    /// Kitty graphics protocol
    Kitty,
}

/// Image placement options
#[derive(Debug, Clone)]
pub struct ImagePlacement {
    /// X position in cells
    pub x: Option<u16>,
    /// Y position in cells
    pub y: Option<u16>,
    /// Width in cells (None = auto)
    pub width: Option<u16>,
    /// Height in cells (None = auto)
    pub height: Option<u16>,
    /// Z-index for layering
    pub z_index: Option<i32>,
}

impl Default for ImagePlacement {
    fn default() -> Self {
        Self {
            x: None,
            y: None,
            width: None,
            height: None,
            z_index: None,
        }
    }
}

impl ImagePlacement {
    /// Create a new placement at the specified position
    pub fn at(x: u16, y: u16) -> Self {
        Self {
            x: Some(x),
            y: Some(y),
            ..Default::default()
        }
    }

    /// Set the width
    pub fn with_width(mut self, width: u16) -> Self {
        self.width = Some(width);
        self
    }

    /// Set the height
    pub fn with_height(mut self, height: u16) -> Self {
        self.height = Some(height);
        self
    }

    /// Set both width and height
    pub fn with_size(mut self, width: u16, height: u16) -> Self {
        self.width = Some(width);
        self.height = Some(height);
        self
    }

    /// Set z-index
    pub fn with_z_index(mut self, z: i32) -> Self {
        self.z_index = Some(z);
        self
    }
}

/// Kitty image protocol builder
pub struct KittyImage<'a> {
    data: &'a [u8],
    format: ImageFormat,
    placement: ImagePlacement,
    image_id: Option<u32>,
    placement_id: Option<u32>,
    width_px: Option<u32>,
    height_px: Option<u32>,
}

impl<'a> KittyImage<'a> {
    /// Create a new Kitty image from raw data
    pub fn new(data: &'a [u8], format: ImageFormat) -> Self {
        Self {
            data,
            format,
            placement: ImagePlacement::default(),
            image_id: None,
            placement_id: None,
            width_px: None,
            height_px: None,
        }
    }

    /// Set placement options
    pub fn placement(mut self, placement: ImagePlacement) -> Self {
        self.placement = placement;
        self
    }

    /// Set image ID for reuse
    pub fn with_image_id(mut self, id: u32) -> Self {
        self.image_id = Some(id);
        self
    }

    /// Set placement ID
    pub fn with_placement_id(mut self, id: u32) -> Self {
        self.placement_id = Some(id);
        self
    }

    /// Set pixel dimensions (required for RGB/RGBA formats)
    pub fn with_pixel_size(mut self, width: u32, height: u32) -> Self {
        self.width_px = Some(width);
        self.height_px = Some(height);
        self
    }

    /// Generate the Kitty protocol escape sequence
    pub fn to_sequence(&self) -> Result<String, std::fmt::Error> {
        // Encode data as base64 first
        let encoded = base64_encode(self.data);

        // Build control data
        let mut control = String::new();

        // Action: transmit and display
        write!(control, "a=T")?;

        // Format
        let format_code = match self.format {
            ImageFormat::Png => 100,
            ImageFormat::Jpeg => 101,
            ImageFormat::Gif => 102,
            ImageFormat::Rgb => 24,
            ImageFormat::Rgba => 32,
        };
        write!(control, ",f={}", format_code)?;

        // Transmission medium: direct
        write!(control, ",t=d")?;

        // Pixel dimensions (required for RGB/RGBA)
        if let Some(w) = self.width_px {
            write!(control, ",s={}", w)?;
        }
        if let Some(h) = self.height_px {
            write!(control, ",v={}", h)?;
        }

        // Image ID
        if let Some(id) = self.image_id {
            write!(control, ",i={}", id)?;
        }

        // Placement ID
        if let Some(id) = self.placement_id {
            write!(control, ",p={}", id)?;
        }

        // Position and size (in cells)
        if let Some(x) = self.placement.x {
            write!(control, ",X={}", x)?;
        }
        if let Some(y) = self.placement.y {
            write!(control, ",Y={}", y)?;
        }
        if let Some(w) = self.placement.width {
            write!(control, ",c={}", w)?;
        }
        if let Some(h) = self.placement.height {
            write!(control, ",r={}", h)?;
        }
        if let Some(z) = self.placement.z_index {
            write!(control, ",z={}", z)?;
        }

        let mut output = String::new();

        // For small images, send in one chunk
        if encoded.len() <= 4096 {
            write!(output, "\x1b_G{};{}\x1b\\", control, encoded)?;
        } else {
            // For large images, chunk the data
            let chunks: Vec<&str> = encoded
                .as_bytes()
                .chunks(4096)
                .map(|chunk| std::str::from_utf8(chunk).unwrap())
                .collect();

            for (i, chunk) in chunks.iter().enumerate() {
                if i == 0 {
                    // First chunk - include control data and set m=1
                    write!(output, "\x1b_G{},m=1;{}\x1b\\", control, chunk)?;
                } else if i == chunks.len() - 1 {
                    // Last chunk - m=0
                    write!(output, "\x1b_Gm=0;{}\x1b\\", chunk)?;
                } else {
                    // Middle chunk - m=1
                    write!(output, "\x1b_Gm=1;{}\x1b\\", chunk)?;
                }
            }
        }

        Ok(output)
    }
}

/// Sixel image encoder
pub struct SixelImage<'a> {
    data: &'a [u8],
    width: u32,
    height: u32,
}

impl<'a> SixelImage<'a> {
    /// Create a new Sixel image from RGB data
    /// Data should be in RGB format (3 bytes per pixel)
    pub fn from_rgb(data: &'a [u8], width: u32, height: u32) -> Self {
        Self {
            data,
            width,
            height,
        }
    }

    /// Generate Sixel escape sequence
    /// This is a simplified implementation that converts RGB to indexed color
    pub fn to_sequence(&self) -> Result<String, std::fmt::Error> {
        let mut output = String::new();

        // Start Sixel sequence: ESC P 0 q
        write!(output, "\x1bP0;0;0q")?;

        // Raster attributes: "Pan;Pad;Ph;Pv
        write!(output, "\"1;1;{};{}", self.width, self.height)?;

        // Define a 8-color palette
        // Colors: Black, Red, Green, Yellow, Blue, Magenta, Cyan, White
        let palette = [
            (0, 0, 0),       // 0: Black
            (100, 0, 0),     // 1: Red
            (0, 100, 0),     // 2: Green
            (100, 100, 0),   // 3: Yellow
            (0, 0, 100),     // 4: Blue
            (100, 0, 100),   // 5: Magenta
            (0, 100, 100),   // 6: Cyan
            (100, 100, 100), // 7: White
        ];

        for (i, (r, g, b)) in palette.iter().enumerate() {
            write!(output, "#{};2;{};{};{}", i, r, g, b)?;
        }

        // Encode image data
        let bytes_per_pixel = 3;
        let stride = self.width as usize * bytes_per_pixel;

        // Process in bands of 6 pixels high (sixel band)
        let num_bands = (self.height as usize + 5) / 6;

        for band in 0..num_bands {
            let band_start = band * 6;

            // For each color in palette
            for color_idx in 0..palette.len() {
                write!(output, "#{}", color_idx)?;

                // Encode one scanline of this band for this color
                for x in 0..self.width as usize {
                    let mut sixel = 0u8;

                    // Check 6 pixels vertically
                    for bit in 0..6 {
                        let y = band_start + bit;
                        if y >= self.height as usize {
                            break;
                        }

                        let offset = y * stride + x * bytes_per_pixel;
                        if offset + 2 < self.data.len() {
                            let r = self.data[offset];
                            let g = self.data[offset + 1];
                            let b = self.data[offset + 2];

                            // Map RGB to closest palette color
                            let pixel_color = match_color_to_palette(r, g, b);

                            if pixel_color == color_idx {
                                sixel |= 1 << bit;
                            }
                        }
                    }

                    // Encode sixel byte (add 63 to make printable)
                    if sixel != 0 {
                        write!(output, "{}", (sixel + 63) as char)?;
                    } else {
                        // Optimization: use '?' for empty sixels
                        write!(output, "?")?;
                    }
                }

                // Carriage return to start of line
                write!(output, "$")?;
            }

            // Next band (line feed)
            if band < num_bands - 1 {
                write!(output, "-")?;
            }
        }

        // End Sixel sequence
        write!(output, "\x1b\\")?;

        Ok(output)
    }
}

/// Match RGB color to closest palette color (8-color)
fn match_color_to_palette(r: u8, g: u8, b: u8) -> usize {
    // Simple threshold-based matching to 8 colors
    let r_bit = if r > 127 { 1 } else { 0 };
    let g_bit = if g > 127 { 2 } else { 0 };
    let b_bit = if b > 127 { 4 } else { 0 };
    (r_bit | g_bit | b_bit) as usize
}

/// Simple base64 encoding
fn base64_encode(data: &[u8]) -> String {
    const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut result = String::new();

    for chunk in data.chunks(3) {
        let mut buf = [0u8; 3];
        for (i, &byte) in chunk.iter().enumerate() {
            buf[i] = byte;
        }

        let b1 = (buf[0] >> 2) as usize;
        let b2 = (((buf[0] & 0x03) << 4) | (buf[1] >> 4)) as usize;
        let b3 = (((buf[1] & 0x0f) << 2) | (buf[2] >> 6)) as usize;
        let b4 = (buf[2] & 0x3f) as usize;

        result.push(CHARS[b1] as char);
        result.push(CHARS[b2] as char);
        result.push(if chunk.len() > 1 {
            CHARS[b3] as char
        } else {
            '='
        });
        result.push(if chunk.len() > 2 {
            CHARS[b4] as char
        } else {
            '='
        });
    }

    result
}

/// Delete a Kitty image by ID
pub fn delete_kitty_image(image_id: u32) -> String {
    format!("\x1b_Ga=d,d=I,i={}\x1b\\", image_id)
}

/// Delete all Kitty images
pub fn delete_all_kitty_images() -> String {
    "\x1b_Ga=d,d=A\x1b\\".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_image_format() {
        assert_eq!(ImageFormat::Png, ImageFormat::Png);
        assert_ne!(ImageFormat::Png, ImageFormat::Jpeg);
    }

    #[test]
    fn test_image_protocol() {
        assert_eq!(ImageProtocol::Kitty, ImageProtocol::Kitty);
        assert_ne!(ImageProtocol::Kitty, ImageProtocol::Sixel);
    }

    #[test]
    fn test_image_placement_default() {
        let placement = ImagePlacement::default();
        assert!(placement.x.is_none());
        assert!(placement.y.is_none());
        assert!(placement.width.is_none());
        assert!(placement.height.is_none());
    }

    #[test]
    fn test_image_placement_builder() {
        let placement = ImagePlacement::at(10, 5).with_size(20, 15).with_z_index(1);

        assert_eq!(placement.x, Some(10));
        assert_eq!(placement.y, Some(5));
        assert_eq!(placement.width, Some(20));
        assert_eq!(placement.height, Some(15));
        assert_eq!(placement.z_index, Some(1));
    }

    #[test]
    fn test_base64_encode() {
        assert_eq!(base64_encode(b"hello"), "aGVsbG8=");
        assert_eq!(base64_encode(b"a"), "YQ==");
        assert_eq!(base64_encode(b"ab"), "YWI=");
        assert_eq!(base64_encode(b"abc"), "YWJj");
    }

    #[test]
    fn test_kitty_image_simple() {
        let data = b"fake image data";
        let img = KittyImage::new(data, ImageFormat::Png);
        let seq = img.to_sequence().unwrap();

        assert!(seq.starts_with("\x1b_G"));
        assert!(seq.contains("a=T"));
        assert!(seq.contains("f=100")); // PNG format
        assert!(seq.contains("t=d")); // Direct transmission
        assert!(seq.ends_with("\x1b\\"));
    }

    #[test]
    fn test_kitty_image_with_placement() {
        let data = b"test";
        let placement = ImagePlacement::at(5, 10).with_size(20, 15);
        let img = KittyImage::new(data, ImageFormat::Jpeg).placement(placement);
        let seq = img.to_sequence().unwrap();

        assert!(seq.contains("X=5"));
        assert!(seq.contains("Y=10"));
        assert!(seq.contains("c=20"));
        assert!(seq.contains("r=15"));
        assert!(seq.contains("f=101")); // JPEG format
    }

    #[test]
    fn test_kitty_image_with_ids() {
        let data = b"test";
        let img = KittyImage::new(data, ImageFormat::Png)
            .with_image_id(42)
            .with_placement_id(7);
        let seq = img.to_sequence().unwrap();

        assert!(seq.contains("i=42"));
        assert!(seq.contains("p=7"));
    }

    #[test]
    fn test_kitty_image_formats() {
        let data = b"test";

        let png = KittyImage::new(data, ImageFormat::Png)
            .to_sequence()
            .unwrap();
        assert!(png.contains("f=100"));

        let jpeg = KittyImage::new(data, ImageFormat::Jpeg)
            .to_sequence()
            .unwrap();
        assert!(jpeg.contains("f=101"));

        let gif = KittyImage::new(data, ImageFormat::Gif)
            .to_sequence()
            .unwrap();
        assert!(gif.contains("f=102"));

        let rgb = KittyImage::new(data, ImageFormat::Rgb)
            .to_sequence()
            .unwrap();
        assert!(rgb.contains("f=24"));

        let rgba = KittyImage::new(data, ImageFormat::Rgba)
            .to_sequence()
            .unwrap();
        assert!(rgba.contains("f=32"));
    }

    #[test]
    fn test_delete_kitty_image() {
        let seq = delete_kitty_image(42);
        assert_eq!(seq, "\x1b_Ga=d,d=I,i=42\x1b\\");
    }

    #[test]
    fn test_delete_all_kitty_images() {
        let seq = delete_all_kitty_images();
        assert_eq!(seq, "\x1b_Ga=d,d=A\x1b\\");
    }

    #[test]
    fn test_sixel_image_creation() {
        let data = vec![255u8; 300]; // 10x10 white image in RGB
        let img = SixelImage::from_rgb(&data, 10, 10);
        assert_eq!(img.width, 10);
        assert_eq!(img.height, 10);
    }

    #[test]
    fn test_sixel_sequence_format() {
        let data = vec![255u8; 12]; // 2x2 white image
        let img = SixelImage::from_rgb(&data, 2, 2);
        let seq = img.to_sequence().unwrap();

        assert!(seq.starts_with("\x1bP0;0;0q"));
        assert!(seq.ends_with("\x1b\\"));
        assert!(seq.contains("\"1;1;2;2")); // Raster attributes
    }

    #[test]
    fn test_kitty_chunking_small_data() {
        let data = b"small";
        let img = KittyImage::new(data, ImageFormat::Png);
        let seq = img.to_sequence().unwrap();

        // Small data should not be chunked
        assert!(!seq.contains("m=1"));
        assert!(!seq.contains("m=0"));
    }
}
