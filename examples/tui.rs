// Interactive TUI Example - Renders Rin terminal buffer in realtime
// Run: cargo run --example tui --features "pty,crossterm"

use crossterm::{
    cursor::{Hide, MoveTo, Show},
    event::{self, Event, KeyCode, KeyModifiers},
    execute,
    style::{Color as CtColor, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
};
use rin::{AnsiParser, Color, Pty, TerminalBuffer};
use std::io::{self, Write};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

fn main() -> anyhow::Result<()> {
    terminal::enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, Hide)?;

    let result = run_terminal(&mut stdout);

    execute!(stdout, Show, LeaveAlternateScreen)?;
    terminal::disable_raw_mode()?;

    if let Err(e) = &result {
        eprintln!("Error: {}", e);
    }

    result
}

fn run_terminal(stdout: &mut io::Stdout) -> anyhow::Result<()> {
    let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string());

    let (mut term_width, mut term_height) = terminal::size()?;
    let (mut width, mut height) = (term_width as usize, term_height.saturating_sub(1) as usize);

    let mut pty = Pty::spawn(&shell, width as u16, height as u16)?;
    let mut buffer = TerminalBuffer::new(width, height);
    let mut parser = AnsiParser::new();

    let (tx, rx) = mpsc::channel::<Vec<u8>>();

    let mut pty_reader = pty.take_reader()?;
    thread::spawn(move || {
        let mut read_buf = [0u8; 4096];
        loop {
            match pty_reader.read(&mut read_buf) {
                Ok(0) => break,
                Ok(n) => {
                    if tx.send(read_buf[..n].to_vec()).is_err() {
                        break;
                    }
                }
                Err(_) => break,
            }
        }
    });

    render_buffer(stdout, &buffer, width, height)?;

    loop {
        if event::poll(Duration::from_millis(16))? {
            match event::read()? {
                Event::Key(key) => {
                    if (key.code == KeyCode::Char('q') || key.code == KeyCode::Char('c'))
                        && key.modifiers.contains(KeyModifiers::CONTROL)
                    {
                        break;
                    }

                    let bytes = key_to_bytes(key.code, key.modifiers);
                    if !bytes.is_empty() {
                        pty.write(&bytes)?;
                    }
                }
                Event::Resize(new_width, new_height) => {
                    if new_width != term_width || new_height != term_height {
                        term_width = new_width;
                        term_height = new_height;
                        width = new_width as usize;
                        height = new_height.saturating_sub(1) as usize;

                        pty.resize(width as u16, height as u16)?;
                        buffer.resize(width, height)?;

                        render_buffer(stdout, &buffer, width, height)?;
                    }
                }
                _ => {}
            }
        }

        let mut needs_render = false;
        while let Ok(data) = rx.try_recv() {
            if let Ok(cmds) = parser.parse(&data) {
                for cmd in cmds {
                    let _ = buffer.execute_command(cmd);
                }
            }

            for response in buffer.drain_responses() {
                let _ = pty.write(&response);
            }

            needs_render = true;
        }

        if needs_render {
            render_buffer(stdout, &buffer, width, height)?;
        }
    }

    Ok(())
}

fn render_buffer(
    stdout: &mut io::Stdout,
    buffer: &TerminalBuffer,
    width: usize,
    height: usize,
) -> anyhow::Result<()> {
    let grid = buffer.grid();
    let (cursor_x, cursor_y) = buffer.cursor_pos();

    for y in 0..grid.height().min(height) {
        execute!(stdout, MoveTo(0, y as u16))?;

        if let Some(row) = grid.row(y) {
            for (x, cell) in row.iter().enumerate() {
                if x >= width {
                    break;
                }
                // Handle reverse video
                let (fg, bg) = if cell.style.reverse {
                    (
                        to_crossterm_color(cell.style.bg),
                        to_crossterm_color(cell.style.fg),
                    )
                } else {
                    (
                        to_crossterm_color(cell.style.fg),
                        to_crossterm_color(cell.style.bg),
                    )
                };

                execute!(
                    stdout,
                    SetForegroundColor(fg),
                    SetBackgroundColor(bg),
                    Print(cell.character)
                )?;
            }
        }

        execute!(stdout, ResetColor)?;
    }

    // Status bar
    let status = format!(
        " {}x{} | Cursor: ({},{}) | Ctrl+Q to exit ",
        width, height, cursor_x, cursor_y
    );
    execute!(
        stdout,
        MoveTo(0, height as u16),
        SetBackgroundColor(CtColor::Blue),
        SetForegroundColor(CtColor::White),
        Print(format!("{:width$}", status, width = width)),
        ResetColor
    )?;

    // Position cursor
    execute!(stdout, MoveTo(cursor_x as u16, cursor_y as u16))?;

    stdout.flush()?;
    Ok(())
}

fn to_crossterm_color(color: Color) -> CtColor {
    CtColor::Rgb {
        r: color.r,
        g: color.g,
        b: color.b,
    }
}

fn key_to_bytes(code: KeyCode, modifiers: KeyModifiers) -> Vec<u8> {
    let ctrl = modifiers.contains(KeyModifiers::CONTROL);

    match code {
        KeyCode::Char(c) => {
            if ctrl && c >= 'a' && c <= 'z' {
                vec![(c as u8) - b'a' + 1]
            } else {
                c.to_string().into_bytes()
            }
        }
        KeyCode::Enter => vec![b'\r'],
        KeyCode::Backspace => vec![0x7f],
        KeyCode::Tab => vec![b'\t'],
        KeyCode::Esc => vec![0x1b],
        KeyCode::Up => vec![0x1b, b'[', b'A'],
        KeyCode::Down => vec![0x1b, b'[', b'B'],
        KeyCode::Right => vec![0x1b, b'[', b'C'],
        KeyCode::Left => vec![0x1b, b'[', b'D'],
        KeyCode::Home => vec![0x1b, b'[', b'H'],
        KeyCode::End => vec![0x1b, b'[', b'F'],
        KeyCode::PageUp => vec![0x1b, b'[', b'5', b'~'],
        KeyCode::PageDown => vec![0x1b, b'[', b'6', b'~'],
        KeyCode::Delete => vec![0x1b, b'[', b'3', b'~'],
        KeyCode::Insert => vec![0x1b, b'[', b'2', b'~'],
        KeyCode::F(n) if n >= 1 && n <= 4 => vec![0x1b, b'O', b'P' + (n - 1)],
        _ => vec![],
    }
}
