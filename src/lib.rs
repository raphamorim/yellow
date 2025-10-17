//! Yellow - A terminal manipulation library
//!
//! Yellow provides a simple API for building terminal user interfaces.
//!
//! # Example
//! ```no_run
//! use yellow::{Screen, Color, Attr};
//!
//! let mut scr = Screen::init()?;
//! scr.mvprint(5, 10, "Hello, World!")?;
//! scr.attron(Attr::BOLD)?;
//! scr.print("Bold text")?;
//! scr.refresh()?;
//! scr.getch()?;
//! scr.endwin()?;
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

mod acs;
mod attr;
mod backend;
mod color;
mod error;
mod image;
mod input;
mod kitty;
mod mosaic;
mod panel;
mod screen;
mod window;

pub use acs::{
    ACS_BLOCK, ACS_BOARD, ACS_BTEE, ACS_BULLET, ACS_CKBOARD, ACS_DARROW, ACS_DEGREE, ACS_DIAMOND,
    ACS_GEQUAL, ACS_HLINE, ACS_LANTERN, ACS_LARROW, ACS_LEQUAL, ACS_LLCORNER, ACS_LRCORNER,
    ACS_LTEE, ACS_NEQUAL, ACS_PI, ACS_PLMINUS, ACS_PLUS, ACS_RARROW, ACS_RTEE, ACS_S1, ACS_S3,
    ACS_S7, ACS_S9, ACS_STERLING, ACS_TTEE, ACS_UARROW, ACS_ULCORNER, ACS_URCORNER, ACS_VLINE,
    AcsChar,
};
pub use attr::Attr;
pub use color::{Color, ColorPair};
pub use error::{Error, Result};
pub use image::{ImageFormat, ImagePlacement, ImageProtocol, KittyImage, SixelImage};
pub use input::Key;
pub use kitty::{KeyEvent, KeyEventType, KittyFlags, Modifiers};
pub use mosaic::{MosaicConfig, SymbolSet, render_mosaic};
pub use panel::Panel;
pub use screen::Screen;
pub use window::Window;
