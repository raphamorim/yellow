use crate::kitty::KeyEvent;

/// Keyboard input key
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Key {
    /// A character key
    Char(char),
    /// Function keys F1-F12
    F(u8),
    /// Arrow keys
    Up,
    Down,
    Left,
    Right,
    /// Special keys
    Enter,
    Backspace,
    Delete,
    Insert,
    Home,
    End,
    PageUp,
    PageDown,
    Tab,
    Escape,
    /// Control + character
    Ctrl(char),
    /// Alt + character
    Alt(char),
    /// Enhanced key event from Kitty keyboard protocol
    Enhanced(KeyEvent),
    /// Unknown/unsupported key
    Unknown,
}

impl Key {
    /// Parse ANSI escape sequence into a Key
    pub(crate) fn from_escape_sequence(seq: &[u8]) -> Option<Self> {
        if seq.is_empty() {
            return None;
        }

        // Simple ESC sequences
        if seq.len() == 1 && seq[0] == 27 {
            return Some(Key::Escape);
        }

        // Check for Kitty keyboard protocol sequence first (CSI ... u)
        if seq.len() >= 4 && seq[0] == 27 && seq[1] == b'[' && seq[seq.len() - 1] == b'u' {
            if let Some(event) = KeyEvent::from_sequence(seq) {
                return Some(Key::Enhanced(event));
            }
        }

        // ESC [ sequences
        if seq.len() >= 3 && seq[0] == 27 && seq[1] == b'[' {
            return match seq[2] {
                b'A' => Some(Key::Up),
                b'B' => Some(Key::Down),
                b'C' => Some(Key::Right),
                b'D' => Some(Key::Left),
                b'H' => Some(Key::Home),
                b'F' => Some(Key::End),
                b'1' if seq.len() >= 4 => match seq[3] {
                    b'~' => Some(Key::Home),
                    b'1'..=b'9' if seq.len() >= 5 && seq[4] == b'~' => {
                        Some(Key::F(seq[3] - b'0' + 10))
                    }
                    _ => None,
                },
                b'2' if seq.len() >= 4 && seq[3] == b'~' => Some(Key::Insert),
                b'3' if seq.len() >= 4 && seq[3] == b'~' => Some(Key::Delete),
                b'4' if seq.len() >= 4 && seq[3] == b'~' => Some(Key::End),
                b'5' if seq.len() >= 4 && seq[3] == b'~' => Some(Key::PageUp),
                b'6' if seq.len() >= 4 && seq[3] == b'~' => Some(Key::PageDown),
                _ => None,
            };
        }

        // ESC O sequences (function keys)
        if seq.len() >= 3 && seq[0] == 27 && seq[1] == b'O' {
            return match seq[2] {
                b'P' => Some(Key::F(1)),
                b'Q' => Some(Key::F(2)),
                b'R' => Some(Key::F(3)),
                b'S' => Some(Key::F(4)),
                _ => None,
            };
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_equality() {
        assert_eq!(Key::Char('a'), Key::Char('a'));
        assert_ne!(Key::Char('a'), Key::Char('b'));
        assert_eq!(Key::Up, Key::Up);
        assert_ne!(Key::Up, Key::Down);
    }

    #[test]
    fn test_escape_sequence_arrow_keys() {
        assert_eq!(Key::from_escape_sequence(&[27, b'[', b'A']), Some(Key::Up));
        assert_eq!(
            Key::from_escape_sequence(&[27, b'[', b'B']),
            Some(Key::Down)
        );
        assert_eq!(
            Key::from_escape_sequence(&[27, b'[', b'C']),
            Some(Key::Right)
        );
        assert_eq!(
            Key::from_escape_sequence(&[27, b'[', b'D']),
            Some(Key::Left)
        );
    }

    #[test]
    fn test_escape_sequence_special_keys() {
        assert_eq!(
            Key::from_escape_sequence(&[27, b'[', b'H']),
            Some(Key::Home)
        );
        assert_eq!(Key::from_escape_sequence(&[27, b'[', b'F']), Some(Key::End));
        assert_eq!(
            Key::from_escape_sequence(&[27, b'[', b'2', b'~']),
            Some(Key::Insert)
        );
        assert_eq!(
            Key::from_escape_sequence(&[27, b'[', b'3', b'~']),
            Some(Key::Delete)
        );
        assert_eq!(
            Key::from_escape_sequence(&[27, b'[', b'5', b'~']),
            Some(Key::PageUp)
        );
        assert_eq!(
            Key::from_escape_sequence(&[27, b'[', b'6', b'~']),
            Some(Key::PageDown)
        );
    }

    #[test]
    fn test_escape_sequence_function_keys() {
        assert_eq!(
            Key::from_escape_sequence(&[27, b'O', b'P']),
            Some(Key::F(1))
        );
        assert_eq!(
            Key::from_escape_sequence(&[27, b'O', b'Q']),
            Some(Key::F(2))
        );
        assert_eq!(
            Key::from_escape_sequence(&[27, b'O', b'R']),
            Some(Key::F(3))
        );
        assert_eq!(
            Key::from_escape_sequence(&[27, b'O', b'S']),
            Some(Key::F(4))
        );
    }

    #[test]
    fn test_escape_sequence_escape_key() {
        assert_eq!(Key::from_escape_sequence(&[27]), Some(Key::Escape));
    }

    #[test]
    fn test_escape_sequence_invalid() {
        assert_eq!(Key::from_escape_sequence(&[]), None);
        assert_eq!(Key::from_escape_sequence(&[27, b'[', b'Z']), None);
        assert_eq!(Key::from_escape_sequence(&[27, b'X']), None);
    }

    #[test]
    fn test_key_variants() {
        let char_key = Key::Char('x');
        let ctrl_key = Key::Ctrl('c');
        let alt_key = Key::Alt('a');
        let func_key = Key::F(5);

        assert!(matches!(char_key, Key::Char('x')));
        assert!(matches!(ctrl_key, Key::Ctrl('c')));
        assert!(matches!(alt_key, Key::Alt('a')));
        assert!(matches!(func_key, Key::F(5)));
    }

    #[test]
    fn test_kitty_protocol_simple() {
        // Simple 'A' key: ESC [ 65 u
        let seq = b"\x1b[65u";
        let key = Key::from_escape_sequence(seq);

        assert!(matches!(key, Some(Key::Enhanced(_))));
        if let Some(Key::Enhanced(event)) = key {
            assert_eq!(event.code, 65);
            assert!(!event.is_ctrl());
            assert!(!event.is_shift());
        }
    }

    #[test]
    fn test_kitty_protocol_with_modifiers() {
        // Ctrl+Shift+A: ESC [ 65 ; 5 u (modifier 1+4=5)
        let seq = b"\x1b[65;5u";
        let key = Key::from_escape_sequence(seq);

        assert!(matches!(key, Some(Key::Enhanced(_))));
        if let Some(Key::Enhanced(event)) = key {
            assert_eq!(event.code, 65);
            assert!(event.is_ctrl());
            assert!(event.is_shift());
            assert!(!event.is_alt());
        }
    }

    #[test]
    fn test_kitty_protocol_with_release() {
        // 'A' release: ESC [ 65 ; 0 ; 3 u
        let seq = b"\x1b[65;0;3u";
        let key = Key::from_escape_sequence(seq);

        assert!(matches!(key, Some(Key::Enhanced(_))));
        if let Some(Key::Enhanced(event)) = key {
            assert_eq!(event.code, 65);
            assert_eq!(event.event_type, crate::kitty::KeyEventType::Release);
        }
    }

    #[test]
    fn test_kitty_protocol_with_repeat() {
        // 'A' repeat: ESC [ 65 ; 0 ; 2 u
        let seq = b"\x1b[65;0;2u";
        let key = Key::from_escape_sequence(seq);

        assert!(matches!(key, Some(Key::Enhanced(_))));
        if let Some(Key::Enhanced(event)) = key {
            assert_eq!(event.code, 65);
            assert_eq!(event.event_type, crate::kitty::KeyEventType::Repeat);
        }
    }

    #[test]
    fn test_kitty_protocol_complex() {
        // Complex sequence with modifiers, event type, and shifted key
        // 'a' with Shift (shifted to 'A'): ESC [ 97 ; 1 ; 1 ; 65 u
        let seq = b"\x1b[97;1;1;65u";
        let key = Key::from_escape_sequence(seq);

        assert!(matches!(key, Some(Key::Enhanced(_))));
        if let Some(Key::Enhanced(event)) = key {
            assert_eq!(event.code, 97);
            assert!(event.is_shift());
            assert_eq!(event.shifted_key, Some(65));
            assert_eq!(event.event_type, crate::kitty::KeyEventType::Press);
        }
    }

    #[test]
    fn test_kitty_protocol_ctrl_alt() {
        // Ctrl+Alt+X: ESC [ 120 ; 6 u (modifier 4+2=6)
        let seq = b"\x1b[120;6u";
        let key = Key::from_escape_sequence(seq);

        assert!(matches!(key, Some(Key::Enhanced(_))));
        if let Some(Key::Enhanced(event)) = key {
            assert_eq!(event.code, 120); // 'x'
            assert!(event.is_ctrl());
            assert!(event.is_alt());
            assert!(!event.is_shift());
        }
    }

    #[test]
    fn test_legacy_sequences_still_work() {
        // Ensure legacy sequences still parse correctly
        assert_eq!(Key::from_escape_sequence(&[27, b'[', b'A']), Some(Key::Up));
        assert_eq!(
            Key::from_escape_sequence(&[27, b'[', b'3', b'~']),
            Some(Key::Delete)
        );
        assert_eq!(
            Key::from_escape_sequence(&[27, b'O', b'P']),
            Some(Key::F(1))
        );
    }
}
