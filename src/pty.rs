use anyhow::{Context, Result};
use portable_pty::{CommandBuilder, MasterPty, PtySize, native_pty_system};
use std::io::{Read, Write};

pub struct Pty {
    master: Box<dyn MasterPty + Send>,
    reader: Box<dyn Read + Send>,
    writer: Box<dyn Write + Send>,
    size: PtySize,
}

impl Pty {
    pub fn spawn(shell: &str, cols: u16, rows: u16) -> Result<Self> {
        let pty_system = native_pty_system();

        let size = PtySize {
            rows,
            cols,
            pixel_width: 0,
            pixel_height: 0,
        };

        let pair = pty_system.openpty(size).context("Failed to open pty")?;

        let mut cmd = CommandBuilder::new(shell);
        cmd.env("TERM", "xterm-256color");

        pair.slave
            .spawn_command(cmd)
            .context("Failed to spawn shell")?;

        let reader = pair
            .master
            .try_clone_reader()
            .context("Failed to clone reader")?;
        let writer = pair.master.take_writer().context("Failed to take writer")?;

        Ok(Self {
            master: pair.master,
            reader,
            writer,
            size,
        })
    }

    pub fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        self.reader.read(buf).context("PTY read failed")
    }

    pub fn write(&mut self, data: &[u8]) -> Result<()> {
        self.writer.write_all(data).context("PTY write failed")?;
        self.writer.flush().context("PTY flush failed")
    }

    pub fn resize(&mut self, cols: u16, rows: u16) -> Result<()> {
        self.size.cols = cols;
        self.size.rows = rows;
        self.master.resize(self.size).context("PTY resize failed")
    }

    pub fn size(&self) -> (u16, u16) {
        (self.size.cols, self.size.rows)
    }
}
