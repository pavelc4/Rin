pub mod screen;
use crate::core::TerminalBuffer;
use anyhow::Result;

pub struct RenderContext<'a> {
    pub buffer: &'a TerminalBuffer,
    pub width: usize,
    pub height: usize,
}

pub trait Renderer {
    fn render(&mut self, context: &RenderContext) -> Result<()>;
}

pub use screen::{ScreenRenderer, AndroidRenderer};
