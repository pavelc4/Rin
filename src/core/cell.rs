use serde::{Deserialize, Serialize};

/// RGB Color representation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl Color {
    pub const BLACK: Color = Color { r: 0, g: 0, b: 0 };
    pub const WHITE: Color = Color { r: 255, g: 255, b: 255 };
    pub const RED: Color = Color { r: 255, g: 0, b: 0 };
    pub const GREEN: Color = Color { r: 0, g: 255, b: 0 };
    pub const BLUE: Color = Color { r: 0, g: 0, b: 255 };

    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }
}

/// Cell style attributes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct CellStyle {
    pub fg: Color,
    pub bg: Color,
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    pub reverse: bool,
}

impl Default for CellStyle {
    fn default() -> Self {
        Self {
            fg: Color::WHITE,
            bg: Color::BLACK,
            bold: false,
            italic: false,
            underline: false,
            reverse: false,
        }
    }
}

/// Single terminal cell
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Cell {
    pub character: char,
    pub style: CellStyle,
}

impl Default for Cell {
    fn default() -> Self {
        Self {
            character: ' ',
            style: CellStyle::default(),
        }
    }
}

impl Cell {
    pub fn new(character: char) -> Self {
        Self {
            character,
            style: CellStyle::default(),
        }
    }

    pub fn with_style(mut self, style: CellStyle) -> Self {
        self.style = style;
        self
    }
}
