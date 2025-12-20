use crate::core::{CellStyle, Color};
use anyhow::Result;
use vte::{Params, Parser, Perform};

#[derive(Debug, Clone, PartialEq)]
pub enum Command {
    Print(char),
    Execute(u8),
    MoveCursor(usize, usize),
    MoveCursorRelative(i32, i32),
    ClearScreen,
    ClearLine,
    SetStyle(CellStyle),
    SetForeground(Color),
    SetBackground(Color),
    SaveCursor,
    RestoreCursor,
    ScrollUp(usize),
    ScrollDown(usize),
    InsertLine(usize),
    DeleteLine(usize),
    EraseChars(usize),
    Reset,
}

pub type ParseResult = Vec<Command>;

pub struct AnsiParser {
    parser: Parser,
    performer: AnsiPerformer,
}

impl AnsiParser {
    pub fn new() -> Self {
        Self {
            parser: Parser::new(),
            performer: AnsiPerformer::new(),
        }
    }

    pub fn parse(&mut self, data: &[u8]) -> Result<ParseResult> {
        self.performer.commands.clear();

        for &byte in data {
            self.parser.advance(&mut self.performer, byte);
        }

        Ok(self.performer.commands.clone())
    }
}

impl Default for AnsiParser {
    fn default() -> Self {
        Self::new()
    }
}

struct AnsiPerformer {
    commands: Vec<Command>,
    current_style: CellStyle,
}

impl AnsiPerformer {
    fn new() -> Self {
        Self {
            commands: Vec::new(),
            current_style: CellStyle::default(),
        }
    }
}

impl Perform for AnsiPerformer {
    fn print(&mut self, c: char) {
        self.commands.push(Command::Print(c));
    }

    fn execute(&mut self, byte: u8) {
        self.commands.push(Command::Execute(byte));
    }

    fn hook(&mut self, _params: &Params, _intermediates: &[u8], _ignore: bool, _c: char) {}

    fn put(&mut self, _byte: u8) {}

    fn unhook(&mut self) {}

    fn osc_dispatch(&mut self, _params: &[&[u8]], _bell_terminated: bool) {}

    fn csi_dispatch(&mut self, params: &Params, _intermediates: &[u8], _ignore: bool, c: char) {
        match c {
            'A' => {
                // Cursor up
                let n = *params.iter().next().and_then(|p| p.first()).unwrap_or(&1) as i32;
                self.commands.push(Command::MoveCursorRelative(0, -n));
            }
            'B' => {
                // Cursor down
                let n = *params.iter().next().and_then(|p| p.first()).unwrap_or(&1) as i32;
                self.commands.push(Command::MoveCursorRelative(0, n));
            }
            'C' => {
                // Cursor forward
                let n = *params.iter().next().and_then(|p| p.first()).unwrap_or(&1) as i32;
                self.commands.push(Command::MoveCursorRelative(n, 0));
            }
            'D' => {
                // Cursor backward
                let n = *params.iter().next().and_then(|p| p.first()).unwrap_or(&1) as i32;
                self.commands.push(Command::MoveCursorRelative(-n, 0));
            }
            'H' | 'f' => {
                // Cursor position
                let mut iter = params.iter();
                let y = iter
                    .next()
                    .and_then(|p| p.first())
                    .unwrap_or(&1)
                    .saturating_sub(1) as usize;
                let x = iter
                    .next()
                    .and_then(|p| p.first())
                    .unwrap_or(&1)
                    .saturating_sub(1) as usize;
                self.commands.push(Command::MoveCursor(x, y));
            }
            'J' => {
                // Erase in display
                let n = params.iter().next().and_then(|p| p.first()).unwrap_or(&0);
                match n {
                    2 => self.commands.push(Command::ClearScreen),
                    _ => {}
                }
            }
            'K' => {
                // Erase in line
                self.commands.push(Command::ClearLine);
            }
            'm' => {
                // SGR - Select Graphic Rendition
                self.handle_sgr(params);
            }
            'L' => {
                // Insert lines
                let n = *params.iter().next().and_then(|p| p.first()).unwrap_or(&1) as usize;
                self.commands.push(Command::InsertLine(n));
            }
            'M' => {
                // Delete lines
                let n = *params.iter().next().and_then(|p| p.first()).unwrap_or(&1) as usize;
                self.commands.push(Command::DeleteLine(n));
            }
            'P' => {
                // Delete characters
                let n = *params.iter().next().and_then(|p| p.first()).unwrap_or(&1) as usize;
                self.commands.push(Command::EraseChars(n));
            }
            'S' => {
                // Scroll up
                let n = *params.iter().next().and_then(|p| p.first()).unwrap_or(&1) as usize;
                self.commands.push(Command::ScrollUp(n));
            }
            'T' => {
                // Scroll down
                let n = *params.iter().next().and_then(|p| p.first()).unwrap_or(&1) as usize;
                self.commands.push(Command::ScrollDown(n));
            }
            's' => {
                // Save cursor
                self.commands.push(Command::SaveCursor);
            }
            'u' => {
                // Restore cursor
                self.commands.push(Command::RestoreCursor);
            }
            _ => {}
        }
    }

    fn esc_dispatch(&mut self, _intermediates: &[u8], _ignore: bool, byte: u8) {
        match byte {
            b'c' => {
                // Reset
                self.commands.push(Command::Reset);
            }
            _ => {}
        }
    }
}

impl AnsiPerformer {
    fn handle_sgr(&mut self, params: &Params) {
        if params.is_empty() {
            // Reset
            self.current_style = CellStyle::default();
            self.commands.push(Command::SetStyle(self.current_style));
            return;
        }

        for param in params.iter() {
            for &p in param {
                match p {
                    0 => {
                        // Reset
                        self.current_style = CellStyle::default();
                    }
                    1 => {
                        // Bold
                        self.current_style.bold = true;
                    }
                    3 => {
                        // Italic
                        self.current_style.italic = true;
                    }
                    4 => {
                        // Underline
                        self.current_style.underline = true;
                    }
                    7 => {
                        // Reverse
                        self.current_style.reverse = true;
                    }
                    22 => {
                        // Not bold
                        self.current_style.bold = false;
                    }
                    23 => {
                        // Not italic
                        self.current_style.italic = false;
                    }
                    24 => {
                        // Not underlined
                        self.current_style.underline = false;
                    }
                    27 => {
                        // Not reversed
                        self.current_style.reverse = false;
                    }
                    30..=37 => {
                        // Foreground color
                        let color = ansi_color(p - 30);
                        self.current_style.fg = color;
                        self.commands.push(Command::SetForeground(color));
                    }
                    40..=47 => {
                        // Background color
                        let color = ansi_color(p - 40);
                        self.current_style.bg = color;
                        self.commands.push(Command::SetBackground(color));
                    }
                    90..=97 => {
                        // Bright foreground
                        let color = ansi_bright_color(p - 90);
                        self.current_style.fg = color;
                        self.commands.push(Command::SetForeground(color));
                    }
                    100..=107 => {
                        // Bright background
                        let color = ansi_bright_color(p - 100);
                        self.current_style.bg = color;
                        self.commands.push(Command::SetBackground(color));
                    }
                    _ => {}
                }
            }
        }

        self.commands.push(Command::SetStyle(self.current_style));
    }
}

fn ansi_color(n: u16) -> Color {
    match n {
        0 => Color::new(0, 0, 0),       // Black
        1 => Color::new(205, 49, 49),   // Red
        2 => Color::new(13, 188, 121),  // Green
        3 => Color::new(229, 229, 16),  // Yellow
        4 => Color::new(36, 114, 200),  // Blue
        5 => Color::new(188, 63, 188),  // Magenta
        6 => Color::new(17, 168, 205),  // Cyan
        7 => Color::new(229, 229, 229), // White
        _ => Color::WHITE,
    }
}

fn ansi_bright_color(n: u16) -> Color {
    match n {
        0 => Color::new(102, 102, 102), // Bright Black
        1 => Color::new(241, 76, 76),   // Bright Red
        2 => Color::new(35, 209, 139),  // Bright Green
        3 => Color::new(245, 245, 67),  // Bright Yellow
        4 => Color::new(59, 142, 234),  // Bright Blue
        5 => Color::new(214, 112, 214), // Bright Magenta
        6 => Color::new(41, 184, 219),  // Bright Cyan
        7 => Color::new(255, 255, 255), // Bright White
        _ => Color::WHITE,
    }
}
