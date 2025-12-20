use super::cell::Cell;
use anyhow::Result;

#[derive(Debug, Clone)]
pub struct Grid {
    cells: Vec<Cell>,
    width: usize,
    height: usize,
}

impl Grid {
    pub fn new(width: usize, height: usize) -> Self {
        let cells = vec![Cell::default(); width * height];
        Self {
            cells,
            width,
            height,
        }
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn get(&self, x: usize, y: usize) -> Option<&Cell> {
        if x >= self.width || y >= self.height {
            return None;
        }
        self.cells.get(y * self.width + x)
    }

    pub fn get_mut(&mut self, x: usize, y: usize) -> Option<&mut Cell> {
        if x >= self.width || y >= self.height {
            return None;
        }
        let idx = y * self.width + x;
        self.cells.get_mut(idx)
    }

    pub fn set(&mut self, x: usize, y: usize, cell: Cell) -> Result<()> {
        if x >= self.width || y >= self.height {
            anyhow::bail!("Position out of bounds: ({}, {})", x, y);
        }
        let idx = y * self.width + x;
        self.cells[idx] = cell;
        Ok(())
    }

    pub fn clear(&mut self) {
        self.cells.fill(Cell::default());
    }

    pub fn resize(&mut self, new_width: usize, new_height: usize) {
        let mut new_cells = vec![Cell::default(); new_width * new_height];

        let copy_width = self.width.min(new_width);
        let copy_height = self.height.min(new_height);

        for y in 0..copy_height {
            for x in 0..copy_width {
                let old_idx = y * self.width + x;
                let new_idx = y * new_width + x;
                new_cells[new_idx] = self.cells[old_idx].clone();
            }
        }

        self.cells = new_cells;
        self.width = new_width;
        self.height = new_height;
    }

    pub fn row(&self, y: usize) -> Option<&[Cell]> {
        if y >= self.height {
            return None;
        }
        let start = y * self.width;
        let end = start + self.width;
        Some(&self.cells[start..end])
    }
}
