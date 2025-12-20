use anyhow::Result;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Key {
    Char(char),
    Enter,
    Backspace,
    Tab,
    Escape,
    Up,
    Down,
    Left,
    Right,
    Home,
    End,
    PageUp,
    PageDown,
    Delete,
    Insert,
    F(u8),
}

#[derive(Debug, Clone, Copy)]
pub struct Modifiers {
    pub ctrl: bool,
    pub alt: bool,
    pub shift: bool,
}

impl Default for Modifiers {
    fn default() -> Self {
        Self {
            ctrl: false,
            alt: false,
            shift: false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct KeyEvent {
    pub key: Key,
    pub modifiers: Modifiers,
}

impl KeyEvent {
    pub fn new(key: Key) -> Self {
        Self {
            key,
            modifiers: Modifiers::default(),
        }
    }

    pub fn with_modifiers(key: Key, modifiers: Modifiers) -> Self {
        Self { key, modifiers }
    }

    pub fn to_ansi(&self) -> Vec<u8> {
        match self.key {
            Key::Char(c) => {
                if self.modifiers.ctrl {
                    if c >= 'a' && c <= 'z' {
                        return vec![(c as u8) - b'a' + 1];
                    } else if c >= 'A' && c <= 'Z' {
                        return vec![(c as u8) - b'A' + 1];
                    }
                }
                c.to_string().into_bytes()
            }
            Key::Enter => vec![b'\r'],
            Key::Backspace => vec![0x7f],
            Key::Tab => vec![b'\t'],
            Key::Escape => vec![0x1b],
            Key::Up => vec![0x1b, b'[', b'A'],
            Key::Down => vec![0x1b, b'[', b'B'],
            Key::Right => vec![0x1b, b'[', b'C'],
            Key::Left => vec![0x1b, b'[', b'D'],
            Key::Home => vec![0x1b, b'[', b'H'],
            Key::End => vec![0x1b, b'[', b'F'],
            Key::PageUp => vec![0x1b, b'[', b'5', b'~'],
            Key::PageDown => vec![0x1b, b'[', b'6', b'~'],
            Key::Delete => vec![0x1b, b'[', b'3', b'~'],
            Key::Insert => vec![0x1b, b'[', b'2', b'~'],
            Key::F(n) => {
                if n >= 1 && n <= 4 {
                    vec![0x1b, b'O', b'P' + (n - 1)]
                } else {
                    vec![]
                }
            }
        }
    }
}

pub struct InputHandler {
    buffer: Vec<u8>,
}

impl InputHandler {
    pub fn new() -> Self {
        Self { buffer: Vec::new() }
    }

    pub fn handle_key(&mut self, event: KeyEvent) -> Result<Vec<u8>> {
        Ok(event.to_ansi())
    }

    pub fn push_bytes(&mut self, bytes: &[u8]) {
        self.buffer.extend_from_slice(bytes);
    }

    pub fn drain(&mut self) -> Vec<u8> {
        self.buffer.drain(..).collect()
    }
}

impl Default for InputHandler {
    fn default() -> Self {
        Self::new()
    }
}
