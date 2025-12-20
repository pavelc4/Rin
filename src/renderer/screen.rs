use super::{RenderContext, Renderer};
use anyhow::Result;

pub struct ScreenRenderer {
    dirty: bool,
}

impl ScreenRenderer {
    pub fn new() -> Self {
        Self { dirty: true }
    }
}

impl Default for ScreenRenderer {
    fn default() -> Self {
        Self::new()
    }
}

impl Renderer for ScreenRenderer {
    fn render(&mut self, context: &RenderContext) -> Result<()> {
        if !self.dirty {
            return Ok(());
        }

        let buffer = context.buffer;
        let grid = buffer.grid();

        for y in 0..context.height {
            if let Some(row) = grid.row(y) {
                for (x, cell) in row.iter().enumerate() {
                    let _ = (x, cell);
                }
            }
        }

        self.dirty = false;
        Ok(())
    }
}

#[warn(dead_code)]
pub struct AndroidRenderer {
    canvas_ptr: Option<usize>,
    font_size: f32,
    dirty: bool,
}

impl AndroidRenderer {
    pub fn new(font_size: f32) -> Self {
        Self {
            canvas_ptr: None,
            font_size,
            dirty: true,
        }
    }

    pub fn set_canvas(&mut self, canvas_ptr: usize) {
        self.canvas_ptr = Some(canvas_ptr);
        self.dirty = true;
    }

    pub fn mark_dirty(&mut self) {
        self.dirty = true;
    }
}

impl Renderer for AndroidRenderer {
    fn render(&mut self, context: &RenderContext) -> Result<()> {
        if !self.dirty || self.canvas_ptr.is_none() {
            return Ok(());
        }

        let buffer = context.buffer;
        let grid = buffer.grid();
        let cursor = buffer.cursor_pos();

        for y in 0..context.height.min(grid.height()) {
            if let Some(row) = grid.row(y) {
                for (x, cell) in row.iter().enumerate() {
                    if x >= context.width {
                        break;
                    }

                    self.render_cell(x, y, cell, cursor == (x, y))?;
                }
            }
        }

        self.dirty = false;
        Ok(())
    }
}

impl AndroidRenderer {
    fn render_cell(
        &self,
        x: usize,
        y: usize,
        cell: &crate::core::Cell,
        is_cursor: bool,
    ) -> Result<()> {
        let _ = (x, y, cell, is_cursor);
        Ok(())
    }
}
