/// Kitty keyboard protocol support
///
/// The kitty keyboard protocol is a progressive enhancement protocol that allows
/// terminals to report more detailed keyboard information.
///
/// Specification: https://sw.kovidgoyal.net/kitty/keyboard-protocol/
use bitflags::bitflags;

bitflags! {
    /// Kitty keyboard protocol flags
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct KittyFlags: u32 {
        /// Disambiguate escape codes (flag 1)
        const DISAMBIGUATE = 1;
        /// Report event types (press, repeat, release) (flag 2)
        const EVENT_TYPES = 2;
        /// Report alternate keys (flag 4)
        const ALTERNATE_KEYS = 4;
        /// Report all keys as escape codes (flag 8)
        const ALL_AS_ESCAPES = 8;
        /// Report associated text (flag 16)
        const REPORT_TEXT = 16;
    }
}

impl Default for KittyFlags {
    fn default() -> Self {
        // Default to basic disambiguation
        KittyFlags::DISAMBIGUATE
    }
}

bitflags! {
    /// Keyboard modifiers
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
    pub struct Modifiers: u8 {
        const SHIFT = 1;
        const ALT = 2;
        const CTRL = 4;
        const SUPER = 8;
        const HYPER = 16;
        const META = 32;
        const CAPS_LOCK = 64;
        const NUM_LOCK = 128;
    }
}

/// Key event type
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyEventType {
    /// Key press
    Press,
    /// Key repeat
    Repeat,
    /// Key release
    Release,
}

impl Default for KeyEventType {
    fn default() -> Self {
        KeyEventType::Press
    }
}

/// Enhanced key event with Kitty protocol data
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct KeyEvent {
    /// The unicode codepoint or key code
    pub code: u32,
    /// Modifiers held during the event
    pub modifiers: Modifiers,
    /// Event type (press, repeat, release)
    pub event_type: KeyEventType,
    /// Shifted key (if different from base)
    pub shifted_key: Option<u32>,
    /// Base layout key (if different from current)
    pub base_key: Option<u32>,
    /// Associated text (if any)
    pub text: Option<String>,
}

impl KeyEvent {
    /// Create a new key event
    pub fn new(code: u32) -> Self {
        Self {
            code,
            ..Default::default()
        }
    }

    /// Create a key event with modifiers
    pub fn with_modifiers(code: u32, modifiers: Modifiers) -> Self {
        Self {
            code,
            modifiers,
            ..Default::default()
        }
    }

    /// Check if Shift is held
    pub fn is_shift(&self) -> bool {
        self.modifiers.contains(Modifiers::SHIFT)
    }

    /// Check if Ctrl is held
    pub fn is_ctrl(&self) -> bool {
        self.modifiers.contains(Modifiers::CTRL)
    }

    /// Check if Alt is held
    pub fn is_alt(&self) -> bool {
        self.modifiers.contains(Modifiers::ALT)
    }

    /// Check if Super is held
    pub fn is_super(&self) -> bool {
        self.modifiers.contains(Modifiers::SUPER)
    }

    /// Parse Kitty keyboard protocol sequence
    /// Format: CSI unicode ; modifiers ; event_type ; shifted_key ; base_layout_key u
    pub(crate) fn from_sequence(seq: &[u8]) -> Option<Self> {
        // Must start with ESC [ and end with 'u'
        if seq.len() < 4 || seq[0] != 27 || seq[1] != b'[' || seq[seq.len() - 1] != b'u' {
            return None;
        }

        // Extract the parameters between [ and u
        let params = &seq[2..seq.len() - 1];
        let params_str = std::str::from_utf8(params).ok()?;

        let parts: Vec<&str> = params_str.split(';').collect();

        if parts.is_empty() {
            return None;
        }

        let code = parts[0].parse::<u32>().ok()?;

        let modifiers = if parts.len() > 1 {
            let mod_val = parts[1].parse::<u8>().ok()?;
            Modifiers::from_bits(mod_val).unwrap_or_default()
        } else {
            Modifiers::empty()
        };

        let event_type = if parts.len() > 2 {
            match parts[2].parse::<u8>().ok()? {
                1 => KeyEventType::Press,
                2 => KeyEventType::Repeat,
                3 => KeyEventType::Release,
                _ => KeyEventType::Press,
            }
        } else {
            KeyEventType::Press
        };

        let shifted_key = if parts.len() > 3 && !parts[3].is_empty() {
            parts[3].parse::<u32>().ok()
        } else {
            None
        };

        let base_key = if parts.len() > 4 && !parts[4].is_empty() {
            parts[4].parse::<u32>().ok()
        } else {
            None
        };

        Some(KeyEvent {
            code,
            modifiers,
            event_type,
            shifted_key,
            base_key,
            text: None,
        })
    }
}

/// Generate escape sequence to enable Kitty keyboard protocol
pub(crate) fn enable_sequence(flags: KittyFlags) -> String {
    format!("\x1b[>{flags}u", flags = flags.bits())
}

/// Generate escape sequence to disable Kitty keyboard protocol
pub(crate) fn disable_sequence() -> String {
    "\x1b[<u".to_string()
}

/// Generate escape sequence to push current keyboard mode and enable new mode
pub(crate) fn push_sequence(flags: KittyFlags) -> String {
    format!("\x1b[>{flags};1u", flags = flags.bits())
}

/// Generate escape sequence to pop keyboard mode
pub(crate) fn pop_sequence() -> String {
    "\x1b[<1u".to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kitty_flags() {
        let flags = KittyFlags::DISAMBIGUATE | KittyFlags::EVENT_TYPES;
        assert!(flags.contains(KittyFlags::DISAMBIGUATE));
        assert!(flags.contains(KittyFlags::EVENT_TYPES));
        assert!(!flags.contains(KittyFlags::ALL_AS_ESCAPES));
        assert_eq!(flags.bits(), 3);
    }

    #[test]
    fn test_default_flags() {
        let flags = KittyFlags::default();
        assert!(flags.contains(KittyFlags::DISAMBIGUATE));
        assert!(!flags.contains(KittyFlags::EVENT_TYPES));
    }

    #[test]
    fn test_modifiers() {
        let mods = Modifiers::CTRL | Modifiers::SHIFT;
        assert!(mods.contains(Modifiers::CTRL));
        assert!(mods.contains(Modifiers::SHIFT));
        assert!(!mods.contains(Modifiers::ALT));
        assert_eq!(mods.bits(), 5);
    }

    #[test]
    fn test_key_event_creation() {
        let event = KeyEvent::new(65); // 'A'
        assert_eq!(event.code, 65);
        assert_eq!(event.modifiers, Modifiers::empty());
        assert_eq!(event.event_type, KeyEventType::Press);
    }

    #[test]
    fn test_key_event_with_modifiers() {
        let event = KeyEvent::with_modifiers(65, Modifiers::CTRL | Modifiers::SHIFT);
        assert_eq!(event.code, 65);
        assert!(event.is_ctrl());
        assert!(event.is_shift());
        assert!(!event.is_alt());
    }

    #[test]
    fn test_parse_simple_sequence() {
        // ESC [ 65 u (just 'A')
        let seq = b"\x1b[65u";
        let event = KeyEvent::from_sequence(seq).unwrap();
        assert_eq!(event.code, 65);
        assert_eq!(event.modifiers, Modifiers::empty());
        assert_eq!(event.event_type, KeyEventType::Press);
    }

    #[test]
    fn test_parse_sequence_with_modifiers() {
        // ESC [ 65 ; 5 u ('A' with Ctrl+Shift, modifier value 1+4=5)
        let seq = b"\x1b[65;5u";
        let event = KeyEvent::from_sequence(seq).unwrap();
        assert_eq!(event.code, 65);
        assert!(event.is_ctrl());
        assert!(event.is_shift());
        assert_eq!(event.event_type, KeyEventType::Press);
    }

    #[test]
    fn test_parse_sequence_with_event_type() {
        // ESC [ 65 ; 5 ; 2 u ('A' with Ctrl+Shift, repeat event)
        let seq = b"\x1b[65;5;2u";
        let event = KeyEvent::from_sequence(seq).unwrap();
        assert_eq!(event.code, 65);
        assert!(event.is_ctrl());
        assert!(event.is_shift());
        assert_eq!(event.event_type, KeyEventType::Repeat);
    }

    #[test]
    fn test_parse_sequence_with_release() {
        // ESC [ 65 ; 0 ; 3 u ('A' release event)
        let seq = b"\x1b[65;0;3u";
        let event = KeyEvent::from_sequence(seq).unwrap();
        assert_eq!(event.code, 65);
        assert_eq!(event.event_type, KeyEventType::Release);
    }

    #[test]
    fn test_parse_sequence_with_shifted_key() {
        // ESC [ 97 ; 1 ; 1 ; 65 u ('a' with shift, shifted to 'A')
        let seq = b"\x1b[97;1;1;65u";
        let event = KeyEvent::from_sequence(seq).unwrap();
        assert_eq!(event.code, 97);
        assert!(event.is_shift());
        assert_eq!(event.shifted_key, Some(65));
    }

    #[test]
    fn test_parse_invalid_sequence() {
        assert!(KeyEvent::from_sequence(b"").is_none());
        assert!(KeyEvent::from_sequence(b"\x1b[").is_none());
        assert!(KeyEvent::from_sequence(b"\x1b[65x").is_none());
        assert!(KeyEvent::from_sequence(b"invalid").is_none());
    }

    #[test]
    fn test_enable_sequence() {
        let seq = enable_sequence(KittyFlags::DISAMBIGUATE);
        assert_eq!(seq, "\x1b[>1u");

        let seq = enable_sequence(KittyFlags::DISAMBIGUATE | KittyFlags::EVENT_TYPES);
        assert_eq!(seq, "\x1b[>3u");
    }

    #[test]
    fn test_disable_sequence() {
        assert_eq!(disable_sequence(), "\x1b[<u");
    }

    #[test]
    fn test_push_pop_sequence() {
        let push = push_sequence(KittyFlags::DISAMBIGUATE | KittyFlags::EVENT_TYPES);
        assert_eq!(push, "\x1b[>3;1u");

        let pop = pop_sequence();
        assert_eq!(pop, "\x1b[<1u");
    }

    #[test]
    fn test_modifiers_combination() {
        // Ctrl+Alt+Shift = 4+2+1 = 7
        let mods = Modifiers::from_bits(7).unwrap();
        assert!(mods.contains(Modifiers::CTRL));
        assert!(mods.contains(Modifiers::ALT));
        assert!(mods.contains(Modifiers::SHIFT));
    }

    #[test]
    fn test_event_type_values() {
        assert_eq!(KeyEventType::Press, KeyEventType::Press);
        assert_ne!(KeyEventType::Press, KeyEventType::Release);
        assert_ne!(KeyEventType::Repeat, KeyEventType::Release);
    }
}
