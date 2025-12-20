//! Interactive terminal demo - type and see the output!
//!
//! Run with: cargo run --example interactive

use rin::{AnsiParser, TerminalBuffer};
use std::io::{self, BufRead, Write};

fn main() {
    println!("╔══════════════════════════════════════════════════════════╗");
    println!("║        Rin Terminal Engine - Interactive Demo            ║");
    println!("╠══════════════════════════════════════════════════════════╣");
    println!("║ Type text or ANSI escape sequences to test the engine.  ║");
    println!("║                                                          ║");
    println!("║ Examples:                                                ║");
    println!("║   hello world          - Plain text                      ║");
    println!("║   \\e[31mRed\\e[0m       - Red text                        ║");
    println!("║   \\e[38;5;196mTest     - 256 color                       ║");
    println!("║   \\e[38;2;255;128;0m   - True color (orange)             ║");
    println!("║   \\e[1mBold\\e[0m       - Bold text                       ║");
    println!("║   \\e[?1049h            - Enter alternate screen          ║");
    println!("║   \\e[?1049l            - Exit alternate screen           ║");
    println!("║   /clear               - Clear buffer                    ║");
    println!("║   /status              - Show buffer status              ║");
    println!("║   /grid                - Show grid contents              ║");
    println!("║   /quit                - Exit                            ║");
    println!("╚══════════════════════════════════════════════════════════╝");
    println!();

    let mut buffer = TerminalBuffer::new(80, 24);
    let mut parser = AnsiParser::new();
    let stdin = io::stdin();

    loop {
        print!("rin> ");
        io::stdout().flush().unwrap();

        let mut line = String::new();
        if stdin.lock().read_line(&mut line).unwrap() == 0 {
            break;
        }

        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        // Handle commands
        match line {
            "/quit" | "/exit" | "/q" => {
                println!("Bye! 👋");
                break;
            }
            "/clear" => {
                buffer.clear();
                println!("✓ Buffer cleared");
                continue;
            }
            "/status" => {
                print_status(&buffer);
                continue;
            }
            "/grid" => {
                print_grid(&buffer);
                continue;
            }
            "/help" => {
                println!("Commands: /clear, /status, /grid, /quit");
                println!("Use \\e for ESC character in ANSI sequences");
                continue;
            }
            _ => {}
        }

        // Convert \e to actual ESC character
        let input = line
            .replace("\\e", "\x1b")
            .replace("\\x1b", "\x1b")
            .replace("\\033", "\x1b")
            .replace("\\n", "\n")
            .replace("\\r", "\r")
            .replace("\\t", "\t");

        // Parse and execute
        match parser.parse(input.as_bytes()) {
            Ok(commands) => {
                println!("  📥 Parsed {} command(s)", commands.len());
                for (i, cmd) in commands.iter().enumerate() {
                    println!("     [{:2}] {:?}", i + 1, cmd);
                }

                for cmd in commands {
                    if let Err(e) = buffer.execute_command(cmd) {
                        println!("  ⚠️  Error: {}", e);
                    }
                }

                // Show current state
                let (x, y) = buffer.cursor_pos();
                println!("  📍 Cursor: ({}, {})", x, y);

                // Show visible text on current line
                if let Some(row) = buffer.grid().row(y) {
                    let text: String = row.iter().map(|c| c.character).collect();
                    let trimmed = text.trim_end();
                    if !trimmed.is_empty() {
                        println!("  📝 Row {}: \"{}\"", y, trimmed);
                    }
                }
            }
            Err(e) => {
                println!("  ❌ Parse error: {}", e);
            }
        }
        println!();
    }
}

fn print_status(buffer: &TerminalBuffer) {
    let (x, y) = buffer.cursor_pos();
    let grid = buffer.grid();

    println!("┌─────────────────────────────┐");
    println!("│      Buffer Status          │");
    println!("├─────────────────────────────┤");
    println!(
        "│ Size:       {:3} x {:3}       │",
        grid.width(),
        grid.height()
    );
    println!("│ Cursor:     ({:3}, {:3})       │", x, y);
    println!("│ Scrollback: {:5} lines     │", buffer.scrollback_len());
    println!("│ Scroll pos: {:5}           │", buffer.scroll_offset());
    println!(
        "│ Alt screen: {:5}           │",
        buffer.is_alternate_screen()
    );
    println!("│ Dirty rows: {:5}           │", grid.has_dirty_rows());
    println!("└─────────────────────────────┘");
}

fn print_grid(buffer: &TerminalBuffer) {
    let grid = buffer.grid();
    let (cx, cy) = buffer.cursor_pos();

    println!("┌{}┐", "─".repeat(grid.width() + 2));

    for y in 0..grid.height().min(10) {
        print!("│ ");
        if let Some(row) = grid.row(y) {
            for (x, cell) in row.iter().enumerate() {
                if x == cx && y == cy {
                    print!("█"); // Cursor position
                } else if cell.character == ' ' {
                    print!("·");
                } else {
                    print!("{}", cell.character);
                }
            }
        }
        println!(" │");
    }

    if grid.height() > 10 {
        println!("│ ... ({} more rows) ... │", grid.height() - 10);
    }

    println!("└{}┘", "─".repeat(grid.width() + 2));
    println!("Legend: █ = cursor, · = empty cell");
}
