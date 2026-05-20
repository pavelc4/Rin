use crate::renderer::AndroidRenderer;
use crate::{Pty, TerminalEngine};
use std::sync::{Arc, Mutex};
use std::thread;

pub struct TerminalSession {
    pub engine: Arc<Mutex<TerminalEngine>>,
    pub pty: Arc<Mutex<Pty>>,
}

impl TerminalSession {
    pub fn new(
        width: usize,
        height: usize,
        font_size: f32,
        home_dir: &str,
        username: &str,
        su_path: Option<&str>,
    ) -> anyhow::Result<Self> {
        let renderer = Box::new(AndroidRenderer::new(font_size));
        let engine = Arc::new(Mutex::new(TerminalEngine::new(width, height, renderer)));

        let shell = su_path.unwrap_or("/system/bin/sh");
        let pty = Arc::new(Mutex::new(Pty::spawn(
            shell,
            width as u16,
            height as u16,
            Some(home_dir),
            Some(username),
        )?));

        let pty_clone = pty.clone();
        let engine_clone = engine.clone();

        thread::spawn(move || {
            let mut buffer = [0u8; 16384];
            let mut reader = {
                let mut pty_guard = pty_clone.lock().unwrap();
                match pty_guard.take_reader() {
                    Ok(r) => r,
                    Err(e) => {
                        log::error!("Failed to take PTY reader: {}", e);
                        return;
                    }
                }
            };

            loop {
                match reader.read(&mut buffer) {
                    Ok(0) => {
                        log::info!("PTY closed (EOF)");
                        break;
                    }
                    Ok(n) => {
                        let mut engine_guard = engine_clone.lock().unwrap();
                        if let Err(e) = engine_guard.write(&buffer[..n]) {
                            log::error!("Failed to write to engine: {}", e);
                        }
                    }
                    Err(e) => {
                        log::error!("Error reading from PTY: {}", e);
                        break;
                    }
                }
            }
        });

        Ok(Self { engine, pty })
    }

    pub fn write(&self, data: &[u8]) -> anyhow::Result<usize> {
        let mut pty = self.pty.lock().unwrap();
        pty.write(data).map(|()| data.len())
    }

    pub fn write_to_engine(&self, data: &[u8]) -> anyhow::Result<()> {
        let mut engine = self.engine.lock().unwrap();
        engine.write(data)
    }

    pub fn render(&self) -> anyhow::Result<()> {
        let mut engine = self.engine.lock().unwrap();
        engine.render()
    }

    pub fn resize(&self, width: usize, height: usize) -> anyhow::Result<()> {
        let mut engine = self.engine.lock().unwrap();
        engine.resize(width, height)?;

        let mut pty = self.pty.lock().unwrap();
        pty.resize(width as u16, height as u16)?;

        Ok(())
    }

    pub fn get_buffer(&self) -> Arc<Mutex<TerminalEngine>> {
        self.engine.clone()
    }
}
