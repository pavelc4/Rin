use crate::core::cell::UnderlineStyle;
use crate::core::{CellStyle, Color, Hyperlink};
use anyhow::Result;
use vte::{Params, Parser, Perform};
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CursorStyle {
    #[default]
    BlinkBlock,
    SteadyBlock,
    BlinkUnderline,
    SteadyUnderline,
    BlinkBar,
    SteadyBar,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Charset {
    #[default]
    Ascii,
    LineDrawing,
}

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
    EraseDisplay(u8),
    EraseLine(u8),
    Reset,
    EnterAlternateScreen,
    ExitAlternateScreen,
    SetTitle(String),
    SetCursorStyle(CursorStyle),
    SetBracketedPaste(bool),
    SetCharset(Charset),
    SetTabStop,
    ClearTabStop,
    ClearAllTabStops,
    DeviceAttributeQuery,
    ShowCursor,
    HideCursor,
    SetHyperlink(Option<Hyperlink>),
    SetScrollRegion { top: usize, bottom: usize },
    SetMouseMode(MouseMode),
    InsertChars(usize),
    DeleteChars(usize),
    Bell,
}

/// Mouse tracking modes
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MouseMode {
    #[default]
    None,
    ReportClick,  // Mode 9 or 1000
    ReportMotion, // Mode 1002
    ReportAll,    // Mode 1003
}

/// Mouse encoding format
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum MouseEncoding {
    #[default]
    X10, // Default
    Utf8, // Mode 1005
    Sgr,  // Mode 1006
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

        self.parser.advance(&mut self.performer, data);

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
        if byte == 0x07 {
            self.commands.push(Command::Bell);
        } else {
            self.commands.push(Command::Execute(byte));
        }
    }

    fn hook(&mut self, _params: &Params, _intermediates: &[u8], _ignore: bool, _c: char) {}

    fn put(&mut self, _byte: u8) {}

    fn unhook(&mut self) {}

    fn osc_dispatch(&mut self, params: &[&[u8]], _bell_terminated: bool) {
        if let Some(cmd) = params.first() {
            match *cmd {
                b"0" | b"2" => {
                    if let Some(title_bytes) = params.get(1) {
                        if let Ok(title) = std::str::from_utf8(title_bytes) {
                            self.commands.push(Command::SetTitle(title.to_string()));
                        }
                    }
                }
                b"8" => {
                    // OSC 8 hyperlink: params[1] = id/params, params[2] = URI
                    let uri = params.get(2).and_then(|b| std::str::from_utf8(b).ok());
                    if let Some(uri) = uri {
                        if uri.is_empty() {
                            // Empty URI = clear hyperlink
                            self.commands.push(Command::SetHyperlink(None));
                        } else {
                            // Parse id from params[1] (format: id=VALUE;...)
                            let id = params.get(1).and_then(|b| {
                                std::str::from_utf8(b).ok().and_then(|s| {
                                    s.split(';').find_map(|kv| kv.strip_prefix("id="))
                                })
                            });
                            let link = Hyperlink::new(id, uri.to_string());
                            self.commands.push(Command::SetHyperlink(Some(link)));
                        }
                    }
                }
                _ => {}
            }
        }
    }

    fn csi_dispatch(&mut self, params: &Params, intermediates: &[u8], _ignore: bool, c: char) {
        if intermediates.first() == Some(&b'?') {
            self.handle_private_mode(params, c);
            return;
        }

        match c {
            'A' => {
                let n = *params.iter().next().and_then(|p| p.first()).unwrap_or(&1) as i32;
                self.commands.push(Command::MoveCursorRelative(0, -n));
            }
            'B' => {
                let n = *params.iter().next().and_then(|p| p.first()).unwrap_or(&1) as i32;
                self.commands.push(Command::MoveCursorRelative(0, n));
            }
            'C' => {
                let n = *params.iter().next().and_then(|p| p.first()).unwrap_or(&1) as i32;
                self.commands.push(Command::MoveCursorRelative(n, 0));
            }
            'D' => {
                let n = *params.iter().next().and_then(|p| p.first()).unwrap_or(&1) as i32;
                self.commands.push(Command::MoveCursorRelative(-n, 0));
            }
            'H' | 'f' => {
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
                let n = *params.iter().next().and_then(|p| p.first()).unwrap_or(&0) as u8;
                if n == 2 {
                    self.commands.push(Command::ClearScreen);
                } else {
                    self.commands.push(Command::EraseDisplay(n));
                }
            }
            'K' => {
                let n = *params.iter().next().and_then(|p| p.first()).unwrap_or(&0) as u8;
                self.commands.push(Command::EraseLine(n));
            }
            'm' => self.handle_sgr(params),
            'L' => {
                let n = *params.iter().next().and_then(|p| p.first()).unwrap_or(&1) as usize;
                self.commands.push(Command::InsertLine(n));
            }
            'M' => {
                let n = *params.iter().next().and_then(|p| p.first()).unwrap_or(&1) as usize;
                self.commands.push(Command::DeleteLine(n));
            }
            '@' => {
                let n = *params.iter().next().and_then(|p| p.first()).unwrap_or(&1) as usize;
                self.commands.push(Command::InsertChars(n));
            }
            'P' => {
                let n = *params.iter().next().and_then(|p| p.first()).unwrap_or(&1) as usize;
                self.commands.push(Command::DeleteChars(n));
            }
            'S' => {
                let n = *params.iter().next().and_then(|p| p.first()).unwrap_or(&1) as usize;
                self.commands.push(Command::ScrollUp(n));
            }
            'T' => {
                let n = *params.iter().next().and_then(|p| p.first()).unwrap_or(&1) as usize;
                self.commands.push(Command::ScrollDown(n));
            }
            's' => self.commands.push(Command::SaveCursor),
            'u' => self.commands.push(Command::RestoreCursor),
            'g' => {
                let n = *params.iter().next().and_then(|p| p.first()).unwrap_or(&0);
                match n {
                    0 => self.commands.push(Command::ClearTabStop),
                    3 => self.commands.push(Command::ClearAllTabStops),
                    _ => {}
                }
            }
            'q' if intermediates.first() == Some(&b' ') => {
                let n = *params.iter().next().and_then(|p| p.first()).unwrap_or(&0);
                let style = match n {
                    0 | 1 => CursorStyle::BlinkBlock,
                    2 => CursorStyle::SteadyBlock,
                    3 => CursorStyle::BlinkUnderline,
                    4 => CursorStyle::SteadyUnderline,
                    5 => CursorStyle::BlinkBar,
                    6 => CursorStyle::SteadyBar,
                    _ => CursorStyle::BlinkBlock,
                };
                self.commands.push(Command::SetCursorStyle(style));
            }
            'c' => {
                self.commands.push(Command::DeviceAttributeQuery);
            }
            'r' => {
                // DECSTBM - Set Top and Bottom Margins
                let mut iter = params.iter();
                let top = iter.next().and_then(|p| p.first().copied()).unwrap_or(1) as usize;
                let bottom = iter.next().and_then(|p| p.first().copied()).unwrap_or(0) as usize;
                // top/bottom are 1-indexed; 0 for bottom means use screen height
                self.commands.push(Command::SetScrollRegion {
                    top: top.saturating_sub(1),
                    bottom: if bottom == 0 {
                        usize::MAX
                    } else {
                        bottom.saturating_sub(1)
                    },
                });
            }
            _ => {}
        }
    }

    fn esc_dispatch(&mut self, intermediates: &[u8], _ignore: bool, byte: u8) {
        if let Some(&intermediate) = intermediates.first() {
            match (intermediate, byte) {
                (b'(', b'0') => {
                    self.commands
                        .push(Command::SetCharset(Charset::LineDrawing));
                    return;
                }
                (b'(', b'B') => {
                    self.commands.push(Command::SetCharset(Charset::Ascii));
                    return;
                }
                _ => {}
            }
        }

        match byte {
            b'c' => self.commands.push(Command::Reset),
            b'7' => self.commands.push(Command::SaveCursor), // DECSC
            b'8' => self.commands.push(Command::RestoreCursor), // DECRC
            b'H' => self.commands.push(Command::SetTabStop), // HTS
            _ => {}
        }
    }
}

impl AnsiPerformer {
    fn handle_private_mode(&mut self, params: &Params, c: char) {
        let mode = params
            .iter()
            .next()
            .and_then(|p| p.first())
            .copied()
            .unwrap_or(0);
        match (mode, c) {
            (1049, 'h') => self.commands.push(Command::EnterAlternateScreen),
            (1049, 'l') => self.commands.push(Command::ExitAlternateScreen),
            (47, 'h') | (1047, 'h') => self.commands.push(Command::EnterAlternateScreen),
            (47, 'l') | (1047, 'l') => self.commands.push(Command::ExitAlternateScreen),
            (2004, 'h') => self.commands.push(Command::SetBracketedPaste(true)),
            (2004, 'l') => self.commands.push(Command::SetBracketedPaste(false)),
            (25, 'h') => self.commands.push(Command::ShowCursor),
            (25, 'l') => self.commands.push(Command::HideCursor),
            // Mouse modes
            (9, 'h') | (1000, 'h') => self
                .commands
                .push(Command::SetMouseMode(MouseMode::ReportClick)),
            (1002, 'h') => self
                .commands
                .push(Command::SetMouseMode(MouseMode::ReportMotion)),
            (1003, 'h') => self
                .commands
                .push(Command::SetMouseMode(MouseMode::ReportAll)),
            (9, 'l') | (1000, 'l') | (1002, 'l') | (1003, 'l') => {
                self.commands.push(Command::SetMouseMode(MouseMode::None))
            }
            _ => {}
        }
    }

    fn handle_sgr(&mut self, params: &Params) {
        if params.is_empty() {
            self.current_style = CellStyle::default();
            self.commands.push(Command::SetStyle(self.current_style));
            return;
        }

        let flat: Vec<u16> = params.iter().flat_map(|p| p.iter().copied()).collect();
        let mut i = 0;

        while i < flat.len() {
            let p = flat[i];
            match p {
                0 => self.current_style = CellStyle::default(),
                1 => self.current_style.bold = true,
                2 => self.current_style.dim = true,
                3 => self.current_style.italic = true,
                4 => {
                    // Check for SGR 4:x subparam
                    if i + 1 < flat.len() && flat[i + 1] <= 5 {
                        let sub = flat[i + 1];
                        self.current_style.underline = match sub {
                            0 => UnderlineStyle::None,
                            1 => UnderlineStyle::Single,
                            2 => UnderlineStyle::Double,
                            3 => UnderlineStyle::Curly,
                            4 => UnderlineStyle::Dotted,
                            5 => UnderlineStyle::Dashed,
                            _ => UnderlineStyle::Single,
                        };
                        i += 1;
                    } else {
                        self.current_style.underline = UnderlineStyle::Single;
                    }
                }
                7 => self.current_style.reverse = true,
                8 => self.current_style.hidden = true,
                9 => self.current_style.strikethrough = true,
                22 => {
                    self.current_style.bold = false;
                    self.current_style.dim = false;
                }
                23 => self.current_style.italic = false,
                24 => self.current_style.underline = UnderlineStyle::None,
                27 => self.current_style.reverse = false,
                28 => self.current_style.hidden = false,
                29 => self.current_style.strikethrough = false,
                30..=37 => {
                    let color = ansi_color(p - 30);
                    self.current_style.fg = color;
                    self.commands.push(Command::SetForeground(color));
                }
                38 => {
                    if let Some(color) = self.parse_extended_color(&flat, &mut i) {
                        self.current_style.fg = color;
                        self.commands.push(Command::SetForeground(color));
                    }
                }
                39 => {
                    self.current_style.fg = Color::WHITE;
                    self.commands.push(Command::SetForeground(Color::WHITE));
                }
                40..=47 => {
                    let color = ansi_color(p - 40);
                    self.current_style.bg = color;
                    self.commands.push(Command::SetBackground(color));
                }
                48 => {
                    if let Some(color) = self.parse_extended_color(&flat, &mut i) {
                        self.current_style.bg = color;
                        self.commands.push(Command::SetBackground(color));
                    }
                }
                49 => {
                    self.current_style.bg = Color::BLACK;
                    self.commands.push(Command::SetBackground(Color::BLACK));
                }
                90..=97 => {
                    let color = ansi_bright_color(p - 90);
                    self.current_style.fg = color;
                    self.commands.push(Command::SetForeground(color));
                }
                100..=107 => {
                    let color = ansi_bright_color(p - 100);
                    self.current_style.bg = color;
                    self.commands.push(Command::SetBackground(color));
                }
                _ => {}
            }
            i += 1;
        }

        self.commands.push(Command::SetStyle(self.current_style));
    }
    fn parse_extended_color(&self, params: &[u16], i: &mut usize) -> Option<Color> {
        let mode = params.get(*i + 1)?;
        match *mode {
            5 => {
                let n = *params.get(*i + 2)? as u8;
                *i += 2;
                Some(color_256(n))
            }
            2 => {
                let r = *params.get(*i + 2)? as u8;
                let g = *params.get(*i + 3)? as u8;
                let b = *params.get(*i + 4)? as u8;
                *i += 4;
                Some(Color::new(r, g, b))
            }
            _ => None,
        }
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

fn color_256(n: u8) -> Color {
    match n {
        0..=15 => {
            if n < 8 {
                ansi_color(n as u16)
            } else {
                ansi_bright_color((n - 8) as u16)
            }
        }
        16..=231 => {
            let n = n - 16;
            let r = (n / 36) % 6;
            let g = (n / 6) % 6;
            let b = n % 6;
            let to_rgb = |v: u8| if v == 0 { 0 } else { 55 + v * 40 };
            Color::new(to_rgb(r), to_rgb(g), to_rgb(b))
        }
        232..=255 => {
            let gray = 8 + (n - 232) * 10;
            Color::new(gray, gray, gray)
        }
    }
}
