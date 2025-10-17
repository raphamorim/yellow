use crate::error::Result;
/// Panel - manages layered windows with z-ordering
///
/// Panels provide a way to manage overlapping windows with automatic
/// z-order handling and efficient updates.
use crate::window::Window;
use std::sync::{Mutex, OnceLock};

static PANEL_STACK: OnceLock<Mutex<Vec<usize>>> = OnceLock::new();

/// A panel wraps a window and provides z-ordering
pub struct Panel {
    window: Window,
    panel_id: usize,
    hidden: bool,
}

impl Panel {
    /// Create a new panel from a window
    pub fn new(window: Window) -> Result<Self> {
        let panel_id = Self::register_panel();

        Ok(Self {
            window,
            panel_id,
            hidden: false,
        })
    }

    /// Get a reference to the window
    pub fn window(&self) -> &Window {
        &self.window
    }

    /// Get a mutable reference to the window
    pub fn window_mut(&mut self) -> &mut Window {
        &mut self.window
    }

    /// Move this panel to the top of the stack
    pub fn top(&mut self) -> Result<()> {
        let stack = PANEL_STACK.get_or_init(|| Mutex::new(Vec::new()));
        let mut guard = stack.lock().unwrap();

        if let Some(pos) = guard.iter().position(|&id| id == self.panel_id) {
            guard.remove(pos);
            guard.push(self.panel_id);
        }

        Ok(())
    }

    /// Move this panel to the bottom of the stack
    pub fn bottom(&mut self) -> Result<()> {
        let stack = PANEL_STACK.get_or_init(|| Mutex::new(Vec::new()));
        let mut guard = stack.lock().unwrap();

        if let Some(pos) = guard.iter().position(|&id| id == self.panel_id) {
            guard.remove(pos);
            guard.insert(0, self.panel_id);
        }

        Ok(())
    }

    /// Hide this panel
    pub fn hide(&mut self) -> Result<()> {
        self.hidden = true;
        Ok(())
    }

    /// Show this panel
    pub fn show(&mut self) -> Result<()> {
        self.hidden = false;
        Ok(())
    }

    /// Check if panel is hidden
    pub fn is_hidden(&self) -> bool {
        self.hidden
    }

    /// Update the panel's window
    pub fn refresh(&mut self) -> Result<()> {
        if !self.hidden {
            self.window.refresh()
        } else {
            Ok(())
        }
    }

    /// Update internal buffer without refreshing
    pub fn wnoutrefresh(&mut self) -> Result<()> {
        if !self.hidden {
            self.window.wnoutrefresh()
        } else {
            Ok(())
        }
    }

    fn register_panel() -> usize {
        let stack = PANEL_STACK.get_or_init(|| Mutex::new(Vec::new()));
        let mut guard = stack.lock().unwrap();

        let id = guard.len();
        guard.push(id);
        id
    }
}

impl Drop for Panel {
    fn drop(&mut self) {
        let stack = PANEL_STACK.get_or_init(|| Mutex::new(Vec::new()));
        let mut guard = stack.lock().unwrap();

        if let Some(pos) = guard.iter().position(|&id| id == self.panel_id) {
            guard.remove(pos);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_panel_creation() {
        let win = Window::new(10, 20, 5, 5).unwrap();
        let panel = Panel::new(win).unwrap();
        assert!(!panel.is_hidden());
    }

    #[test]
    fn test_panel_hide_show() {
        let win = Window::new(10, 20, 5, 5).unwrap();
        let mut panel = Panel::new(win).unwrap();

        assert!(!panel.is_hidden());

        panel.hide().unwrap();
        assert!(panel.is_hidden());

        panel.show().unwrap();
        assert!(!panel.is_hidden());
    }

    #[test]
    fn test_panel_window_access() {
        let win = Window::new(10, 20, 5, 5).unwrap();
        let mut panel = Panel::new(win).unwrap();

        assert_eq!(panel.window().get_size(), (10, 20));
        assert_eq!(panel.window().get_position(), (5, 5));

        // Test mutable access
        panel.window_mut().set_fg(crate::color::Color::Red).unwrap();
    }

    #[test]
    fn test_panel_z_order() {
        let win1 = Window::new(10, 20, 0, 0).unwrap();
        let win2 = Window::new(10, 20, 5, 5).unwrap();

        let mut panel1 = Panel::new(win1).unwrap();
        let mut panel2 = Panel::new(win2).unwrap();

        panel1.top().unwrap();
        panel2.bottom().unwrap();

        // Just verify no panics
        assert_eq!(panel1.panel_id, 0);
        assert_eq!(panel2.panel_id, 1);
    }
}
