use rin::{AnsiParser, Pty, TerminalBuffer};
use std::io::{self, Read, Write};
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

fn main() {
    println!("Rin Terminal - Shell Demo");
    println!("Type commands. Press Ctrl+D to exit.\n");

    let shell = std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string());

    let mut pty = match Pty::spawn(&shell, 80, 24) {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Failed to spawn shell: {}", e);
            return;
        }
    };

    let mut buffer = TerminalBuffer::new(80, 24);
    let mut parser = AnsiParser::new();

    let (tx, rx) = mpsc::channel::<u8>();

    thread::spawn(move || {
        let stdin = io::stdin();
        for byte in stdin.lock().bytes() {
            if let Ok(b) = byte {
                if tx.send(b).is_err() {
                    break;
                }
            }
        }
    });

    let mut read_buf = [0u8; 4096];

    loop {
        while let Ok(b) = rx.try_recv() {
            if let Err(e) = pty.write(&[b]) {
                eprintln!("Write error: {}", e);
                return;
            }
        }

        match pty.read(&mut read_buf) {
            Ok(0) => break,
            Ok(n) => {
                let data = &read_buf[..n];

                io::stdout().write_all(data).unwrap();
                io::stdout().flush().unwrap();

                if let Ok(cmds) = parser.parse(data) {
                    for cmd in cmds {
                        let _ = buffer.execute_command(cmd);
                    }
                }
            }
            Err(_) => {
                thread::sleep(Duration::from_millis(10));
            }
        }
    }

    println!("\nShell exited.");
}
