use super::cell::{Cell, CellStyle};
use super::grid::Grid;
use crate::parser::{Charset, Command, CursorStyle};
use anyhow::Result;
use std::collections::VecDeque;

const DEFAULT_SCROLLBACK_LIMIT: usize = 10_000;

#[derive(Debug, Clone)]
pub struct TerminalBuffer {
    grid: Grid,
    cursor_x: usize,
    cursor_y: usize,
    current_style: CellStyle,
    saved_cursor: Option<(usize, usize, CellStyle)>,
    scrollback: VecDeque<Vec<Cell>>,
    scrollback_limit: usize,
    scroll_offset: usize,
    alternate_state: Option<Box<AlternateState>>,
    cursor_style: CursorStyle,
    bracketed_paste: bool,
    charset: Charset,
    tab_stops: Vec<bool>,
    pending_responses: Vec<Vec<u8>>,
}

#[derive(Debug, Clone)]
struct AlternateState {
    grid: Grid,
    cursor_x: usize,
    cursor_y: usize,
    current_style: CellStyle,
    scrollback: VecDeque<Vec<Cell>>,
}

impl TerminalBuffer {
    pub fn new(width: usize, height: usize) -> Self {
        let mut tab_stops = vec![false; width];
        for i in (8..width).step_by(8) {
            tab_stops[i] = true;
        }

        Self {
            grid: Grid::new(width, height),
            cursor_x: 0,
            cursor_y: 0,
            current_style: CellStyle::default(),
            saved_cursor: None,
            scrollback: VecDeque::new(),
            scrollback_limit: DEFAULT_SCROLLBACK_LIMIT,
            scroll_offset: 0,
            alternate_state: None,
            cursor_style: CursorStyle::default(),
            bracketed_paste: false,
            charset: Charset::default(),
            tab_stops,
            pending_responses: Vec::new(),
        }
    }

    pub fn grid(&self) -> &Grid {
        &self.grid
    }

    pub fn cursor_pos(&self) -> (usize, usize) {
        (self.cursor_x, self.cursor_y)
    }

    pub fn current_style(&self) -> CellStyle {
        self.current_style
    }

    pub fn scrollback_len(&self) -> usize {
        self.scrollback.len()
    }

    pub fn scroll_offset(&self) -> usize {
        self.scroll_offset
    }

    pub fn scroll_by(&mut self, delta: i32) {
        let new_offset = (self.scroll_offset as i32 + delta)
            .max(0)
            .min(self.scrollback.len() as i32) as usize;
        self.scroll_offset = new_offset;
    }

    pub fn scroll_to(&mut self, offset: usize) {
        self.scroll_offset = offset.min(self.scrollback.len());
    }

    pub fn scroll_to_bottom(&mut self) {
        self.scroll_offset = 0;
    }

    pub fn scrollback_row(&self, index: usize) -> Option<&[Cell]> {
        self.scrollback.get(index).map(|v| v.as_slice())
    }

    pub fn set_scrollback_limit(&mut self, limit: usize) {
        self.scrollback_limit = limit;
        while self.scrollback.len() > limit {
            self.scrollback.pop_front();
        }
    }

    pub fn is_alternate_screen(&self) -> bool {
        self.alternate_state.is_some()
    }

    pub fn cursor_style(&self) -> CursorStyle {
        self.cursor_style
    }

    pub fn is_bracketed_paste(&self) -> bool {
        self.bracketed_paste
    }

    pub fn charset(&self) -> Charset {
        self.charset
    }

    pub fn drain_responses(&mut self) -> Vec<Vec<u8>> {
        std::mem::take(&mut self.pending_responses)
    }

    fn translate_char(&self, c: char) -> char {
        if self.charset == Charset::LineDrawing {
            match c {
                'j' => '┘',
                'k' => '┐',
                'l' => '┌',
                'm' => '└',
                'n' => '┼',
                'q' => '─',
                't' => '├',
                'u' => '┤',
                'v' => '┴',
                'w' => '┬',
                'x' => '│',
                'a' => '▒',
                _ => c,
            }
        } else {
            c
        }
    }

    pub fn write_char(&mut self, c: char) -> Result<()> {
        let translated = self.translate_char(c);
        if let Some(cell) = self.grid.get_mut(self.cursor_x, self.cursor_y) {
            cell.character = translated;
            cell.style = self.current_style;
        }

        self.cursor_x += 1;
        if self.cursor_x >= self.grid.width() {
            self.cursor_x = 0;
            self.cursor_y += 1;
            if self.cursor_y >= self.grid.height() {
                self.scroll_up(1);
            }
        }

        Ok(())
    }

    fn advance_to_next_tab_stop(&mut self) {
        let width = self.grid.width();
        for x in (self.cursor_x + 1)..width {
            if self.tab_stops.get(x).copied().unwrap_or(false) {
                self.cursor_x = x;
                return;
            }
        }
        self.cursor_x = width.saturating_sub(1);
    }

    fn scroll_up(&mut self, n: usize) {
        let width = self.grid.width();
        let height = self.grid.height();

        for y in 0..n.min(height) {
            if let Some(row) = self.grid.row(y) {
                self.scrollback.push_back(row.to_vec());
            }
        }

        while self.scrollback.len() > self.scrollback_limit {
            self.scrollback.pop_front();
        }

        for y in n..height {
            for x in 0..width {
                if let Some(cell) = self.grid.get(x, y).cloned() {
                    let _ = self.grid.set(x, y - n, cell);
                }
            }
        }

        for y in (height.saturating_sub(n))..height {
            for x in 0..width {
                let _ = self.grid.set(x, y, Cell::default());
            }
        }

        self.cursor_y = self.cursor_y.saturating_sub(n);
    }

    fn scroll_down(&mut self, n: usize) {
        let width = self.grid.width();
        let height = self.grid.height();

        for y in (0..(height - n)).rev() {
            for x in 0..width {
                if let Some(cell) = self.grid.get(x, y).cloned() {
                    let _ = self.grid.set(x, y + n, cell);
                }
            }
        }

        for y in 0..n.min(height) {
            for x in 0..width {
                let _ = self.grid.set(x, y, Cell::default());
            }
        }
    }

    pub fn execute_command(&mut self, cmd: Command) -> Result<()> {
        match cmd {
            Command::Print(c) => {
                if c == '\n' {
                    self.cursor_x = 0;
                    self.cursor_y += 1;
                    if self.cursor_y >= self.grid.height() {
                        self.scroll_up(1);
                    }
                } else if c == '\r' {
                    self.cursor_x = 0;
                } else if c == '\t' {
                    self.advance_to_next_tab_stop();
                } else {
                    self.write_char(c)?;
                }
            }
            Command::Execute(byte) => match byte {
                b'\n' => {
                    self.cursor_x = 0;
                    self.cursor_y += 1;
                    if self.cursor_y >= self.grid.height() {
                        self.scroll_up(1);
                    }
                }
                b'\r' => self.cursor_x = 0,
                b'\t' => {
                    let tab_stop = 8;
                    self.cursor_x = ((self.cursor_x / tab_stop) + 1) * tab_stop;
                }
                0x08 => {
                    if self.cursor_x > 0 {
                        self.cursor_x -= 1;
                    }
                }
                _ => {}
            },
            Command::MoveCursor(x, y) => {
                self.cursor_x = x.min(self.grid.width() - 1);
                self.cursor_y = y.min(self.grid.height() - 1);
            }
            Command::MoveCursorRelative(dx, dy) => {
                self.cursor_x = (self.cursor_x as i32 + dx)
                    .max(0)
                    .min(self.grid.width() as i32 - 1) as usize;
                self.cursor_y = (self.cursor_y as i32 + dy)
                    .max(0)
                    .min(self.grid.height() as i32 - 1) as usize;
            }
            Command::ClearScreen => {
                self.grid.clear();
                self.cursor_x = 0;
                self.cursor_y = 0;
            }
            Command::ClearLine => {
                for x in 0..self.grid.width() {
                    let _ = self.grid.set(x, self.cursor_y, Cell::default());
                }
            }
            Command::EraseDisplay(mode) => {
                let width = self.grid.width();
                let height = self.grid.height();
                match mode {
                    0 => {
                        for x in self.cursor_x..width {
                            let _ = self.grid.set(x, self.cursor_y, Cell::default());
                        }
                        for y in (self.cursor_y + 1)..height {
                            for x in 0..width {
                                let _ = self.grid.set(x, y, Cell::default());
                            }
                        }
                    }
                    1 => {
                        for y in 0..self.cursor_y {
                            for x in 0..width {
                                let _ = self.grid.set(x, y, Cell::default());
                            }
                        }
                        for x in 0..=self.cursor_x.min(width.saturating_sub(1)) {
                            let _ = self.grid.set(x, self.cursor_y, Cell::default());
                        }
                    }
                    _ => {}
                }
            }
            Command::EraseLine(mode) => {
                let width = self.grid.width();
                match mode {
                    0 => {
                        for x in self.cursor_x..width {
                            let _ = self.grid.set(x, self.cursor_y, Cell::default());
                        }
                    }
                    1 => {
                        for x in 0..=self.cursor_x.min(width.saturating_sub(1)) {
                            let _ = self.grid.set(x, self.cursor_y, Cell::default());
                        }
                    }
                    2 => {
                        for x in 0..width {
                            let _ = self.grid.set(x, self.cursor_y, Cell::default());
                        }
                    }
                    _ => {}
                }
            }
            Command::SetStyle(style) => {
                self.current_style = style;
            }
            Command::SetForeground(color) => {
                self.current_style.fg = color;
            }
            Command::SetBackground(color) => {
                self.current_style.bg = color;
            }
            Command::SaveCursor => {
                self.saved_cursor = Some((self.cursor_x, self.cursor_y, self.current_style));
            }
            Command::RestoreCursor => {
                if let Some((x, y, style)) = self.saved_cursor {
                    self.cursor_x = x;
                    self.cursor_y = y;
                    self.current_style = style;
                }
            }
            Command::ScrollUp(n) => {
                self.scroll_up(n);
            }
            Command::ScrollDown(n) => {
                self.scroll_down(n);
            }
            Command::InsertLine(n) => {
                self.scroll_down(n);
            }
            Command::DeleteLine(n) => {
                self.scroll_up(n);
            }
            Command::EraseChars(n) => {
                for i in 0..n {
                    if self.cursor_x + i < self.grid.width() {
                        let _ = self
                            .grid
                            .set(self.cursor_x + i, self.cursor_y, Cell::default());
                    }
                }
            }
            Command::Reset => {
                self.grid.clear();
                self.cursor_x = 0;
                self.cursor_y = 0;
                self.current_style = CellStyle::default();
                self.saved_cursor = None;
            }
            Command::EnterAlternateScreen => {
                self.enter_alternate_screen();
            }
            Command::ExitAlternateScreen => {
                self.exit_alternate_screen();
            }
            Command::SetTitle(_title) => {}
            Command::SetCursorStyle(style) => {
                self.cursor_style = style;
            }
            Command::SetBracketedPaste(enabled) => {
                self.bracketed_paste = enabled;
            }
            Command::SetCharset(charset) => {
                self.charset = charset;
            }
            Command::SetTabStop => {
                if self.cursor_x < self.tab_stops.len() {
                    self.tab_stops[self.cursor_x] = true;
                }
            }
            Command::ClearTabStop => {
                if self.cursor_x < self.tab_stops.len() {
                    self.tab_stops[self.cursor_x] = false;
                }
            }
            Command::ClearAllTabStops => {
                self.tab_stops.fill(false);
            }
            Command::ShowCursor | Command::HideCursor => {}
            Command::DeviceAttributeQuery => {
                self.pending_responses.push(b"\x1b[?1;2c".to_vec());
            }
        }
        Ok(())
    }

    pub fn resize(&mut self, width: usize, height: usize) -> Result<()> {
        self.grid.resize(width, height);
        self.cursor_x = self.cursor_x.min(width.saturating_sub(1));
        self.cursor_y = self.cursor_y.min(height.saturating_sub(1));
        Ok(())
    }

    pub fn clear(&mut self) {
        self.grid.clear();
        self.cursor_x = 0;
        self.cursor_y = 0;
    }
    pub fn enter_alternate_screen(&mut self) {
        if self.alternate_state.is_some() {
            return;
        }

        let width = self.grid.width();
        let height = self.grid.height();

        let state = AlternateState {
            grid: std::mem::replace(&mut self.grid, Grid::new(width, height)),
            cursor_x: self.cursor_x,
            cursor_y: self.cursor_y,
            current_style: self.current_style,
            scrollback: std::mem::take(&mut self.scrollback),
        };

        self.alternate_state = Some(Box::new(state));
        self.cursor_x = 0;
        self.cursor_y = 0;
        self.current_style = CellStyle::default();
    }

    pub fn exit_alternate_screen(&mut self) {
        if let Some(state) = self.alternate_state.take() {
            self.grid = state.grid;
            self.cursor_x = state.cursor_x;
            self.cursor_y = state.cursor_y;
            self.current_style = state.current_style;
            self.scrollback = state.scrollback;
        }
    }
}
