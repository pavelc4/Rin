use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// RGB Color representation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Color {
    pub const BLACK: Color = Color { r: 0, g: 0, b: 0 };
    pub const WHITE: Color = Color {
        r: 255,
        g: 255,
        b: 255,
    };
    pub const RED: Color = Color { r: 255, g: 0, b: 0 };
    pub const GREEN: Color = Color { r: 0, g: 255, b: 0 };
    pub const BLUE: Color = Color { r: 0, g: 0, b: 255 };

    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }
}

/// Hyperlink for OSC 8 support
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Hyperlink {
    inner: Arc<HyperlinkInner>,
}

#[derive(Debug, PartialEq, Eq)]
struct HyperlinkInner {
    id: String,
    uri: String,
}

impl Hyperlink {
    pub fn new(id: Option<&str>, uri: String) -> Self {
        let id = id.map(|s| s.to_string()).unwrap_or_else(|| {
            use std::sync::atomic::{AtomicU32, Ordering};
            static COUNTER: AtomicU32 = AtomicU32::new(0);
            format!("{}_rin", COUNTER.fetch_add(1, Ordering::Relaxed))
        });
        Self {
            inner: Arc::new(HyperlinkInner { id, uri }),
        }
    }

    pub fn id(&self) -> &str {
        &self.inner.id
    }

    pub fn uri(&self) -> &str {
        &self.inner.uri
    }
}

/// Underline style variants
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum UnderlineStyle {
    #[default]
    None,
    Single,
    Double,
    Curly,
    Dotted,
    Dashed,
}

/// Cell style attributes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct CellStyle {
    pub fg: Color,
    pub bg: Color,
    pub bold: bool,
    pub italic: bool,
    pub underline: UnderlineStyle,
    pub reverse: bool,
    pub strikethrough: bool,
    pub dim: bool,
    pub hidden: bool,
}

impl Default for CellStyle {
    fn default() -> Self {
        Self {
            fg: Color::WHITE,
            bg: Color::BLACK,
            bold: false,
            italic: false,
            underline: UnderlineStyle::None,
            reverse: false,
            strikethrough: false,
            dim: false,
            hidden: false,
        }
    }
}

/// Single terminal cell
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Cell {
    pub character: char,
    pub style: CellStyle,
    #[serde(skip)]
    pub hyperlink: Option<Hyperlink>,
    /// Zero-width combining characters attached to this cell
    #[serde(skip)]
    pub zerowidth: Vec<char>,
}

impl Default for Cell {
    fn default() -> Self {
        Self {
            character: ' ',
            style: CellStyle::default(),
            hyperlink: None,
            zerowidth: Vec::new(),
        }
    }
}

impl Cell {
    pub fn new(character: char) -> Self {
        Self {
            character,
            style: CellStyle::default(),
            hyperlink: None,
            zerowidth: Vec::new(),
        }
    }

    /// Push a zero-width character (combining char, emoji joiner, etc.)
    pub fn push_zerowidth(&mut self, c: char) {
        self.zerowidth.push(c);
    }

    pub fn with_style(mut self, style: CellStyle) -> Self {
        self.style = style;
        self
    }

    pub fn with_hyperlink(mut self, hyperlink: Option<Hyperlink>) -> Self {
        self.hyperlink = hyperlink;
        self
    }
}
