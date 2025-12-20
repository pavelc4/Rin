use rin::{AnsiParser, TerminalBuffer};
use std::io::{self, BufRead, Write};

fn main() {
    println!("Rin Terminal Engine - Interactive Demo");
    println!("---------------------------------------");
    println!("Type text or ANSI sequences. Use \\e for ESC.");
    println!("Commands: /status, /grid, /clear, /quit\n");

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

        match line {
            "/quit" | "/exit" | "/q" => break,
            "/clear" => {
                buffer.clear();
                println!("Buffer cleared");
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
            _ => {}
        }

        let input = line
            .replace("\\e", "\x1b")
            .replace("\\n", "\n")
            .replace("\\r", "\r")
            .replace("\\t", "\t");

        match parser.parse(input.as_bytes()) {
            Ok(commands) => {
                println!("  Parsed {} command(s)", commands.len());
                for (i, cmd) in commands.iter().enumerate() {
                    println!("  [{}] {:?}", i + 1, cmd);
                }

                for cmd in commands {
                    if let Err(e) = buffer.execute_command(cmd) {
                        println!("  Error: {}", e);
                    }
                }

                let (x, y) = buffer.cursor_pos();
                println!("  Cursor: ({}, {})", x, y);
            }
            Err(e) => println!("  Parse error: {}", e),
        }
        println!();
    }
}

fn print_status(buffer: &TerminalBuffer) {
    let (x, y) = buffer.cursor_pos();
    let grid = buffer.grid();

    println!("Size:       {} x {}", grid.width(), grid.height());
    println!("Cursor:     ({}, {})", x, y);
    println!("Scrollback: {} lines", buffer.scrollback_len());
    println!("Alt screen: {}", buffer.is_alternate_screen());
    println!("Has dirty:  {}", grid.has_dirty_rows());
}

fn print_grid(buffer: &TerminalBuffer) {
    let grid = buffer.grid();
    let (cx, cy) = buffer.cursor_pos();

    for y in 0..grid.height().min(10) {
        print!("{:2}| ", y);
        if let Some(row) = grid.row(y) {
            for (x, cell) in row.iter().enumerate() {
                if x == cx && y == cy {
                    print!("_");
                } else if cell.character == ' ' {
                    print!(".");
                } else {
                    print!("{}", cell.character);
                }
            }
        }
        println!();
    }

    if grid.height() > 10 {
        println!("... {} more rows", grid.height() - 10);
    }
}
