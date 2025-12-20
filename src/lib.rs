pub mod core;
pub mod input;
pub mod parser;
pub mod renderer;

#[cfg(test)]
mod tests;

pub use core::{Cell, CellStyle, Color, Grid, TerminalBuffer};
pub use input::{InputHandler, Key, KeyEvent, Modifiers};
pub use parser::{AnsiParser, Command, ParseResult};
pub use renderer::{AndroidRenderer, RenderContext, Renderer, ScreenRenderer};

use anyhow::Result;

pub struct TerminalEngine {
    buffer: TerminalBuffer,
    parser: AnsiParser,
    renderer: Box<dyn Renderer>,
    width: usize,
    height: usize,
}

impl TerminalEngine {
    pub fn new(width: usize, height: usize, renderer: Box<dyn Renderer>) -> Self {
        Self {
            buffer: TerminalBuffer::new(width, height),
            parser: AnsiParser::new(),
            renderer,
            width,
            height,
        }
    }

    pub fn write(&mut self, data: &[u8]) -> Result<()> {
        let commands = self.parser.parse(data)?;

        for cmd in commands {
            self.buffer.execute_command(cmd)?;
        }

        Ok(())
    }

    pub fn render(&mut self) -> Result<()> {
        let context = RenderContext {
            buffer: &self.buffer,
            width: self.width,
            height: self.height,
        };

        self.renderer.render(&context)
    }

    pub fn resize(&mut self, width: usize, height: usize) -> Result<()> {
        self.width = width;
        self.height = height;
        self.buffer.resize(width, height)
    }

    pub fn buffer(&self) -> &TerminalBuffer {
        &self.buffer
    }

    pub fn clear(&mut self) {
        self.buffer.clear();
    }
}

#[cfg(feature = "android")]
pub mod android;
