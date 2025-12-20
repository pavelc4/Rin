use super::cell::{Cell, CellStyle};
use super::grid::Grid;
use crate::parser::Command;
use anyhow::Result;

#[derive(Debug, Clone)]
pub struct TerminalBuffer {
    grid: Grid,
    cursor_x: usize,
    cursor_y: usize,
    current_style: CellStyle,
    saved_cursor: Option<(usize, usize, CellStyle)>,
}

impl TerminalBuffer {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            grid: Grid::new(width, height),
            cursor_x: 0,
            cursor_y: 0,
            current_style: CellStyle::default(),
            saved_cursor: None,
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

    pub fn write_char(&mut self, c: char) -> Result<()> {
        if let Some(cell) = self.grid.get_mut(self.cursor_x, self.cursor_y) {
            cell.character = c;
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

    fn scroll_up(&mut self, n: usize) {
        let width = self.grid.width();
        let height = self.grid.height();

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

        self.cursor_y = self.cursor_y.saturating_sub(n).max(0);
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
                    let tab_stop = 8;
                    self.cursor_x = ((self.cursor_x / tab_stop) + 1) * tab_stop;
                    if self.cursor_x >= self.grid.width() {
                        self.cursor_x = 0;
                        self.cursor_y += 1;
                        if self.cursor_y >= self.grid.height() {
                            self.scroll_up(1);
                        }
                    }
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
}
