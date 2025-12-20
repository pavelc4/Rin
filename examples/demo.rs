use rin::{AnsiParser, TerminalBuffer};

fn main() {
    println!("=== Rin Terminal Engine Demo ===\n");

    let mut buffer = TerminalBuffer::new(40, 10);
    let mut parser = AnsiParser::new();

    println!("Demo 1: Basic Text");
    write_and_execute(&mut buffer, &mut parser, b"Hello, Rin Terminal!\n");
    print_buffer_state(&buffer);

    println!("\nDemo 2: Basic ANSI Colors");
    write_and_execute(
        &mut buffer,
        &mut parser,
        b"\x1b[31mRed\x1b[0m \x1b[32mGreen\x1b[0m \x1b[34mBlue\x1b[0m\n",
    );
    print_buffer_state(&buffer);

    println!("\nDemo 3: 256 Color Palette");
    write_and_execute(
        &mut buffer,
        &mut parser,
        b"\x1b[38;5;196mRed256\x1b[0m \x1b[38;5;46mGreen256\x1b[0m\n",
    );
    print_buffer_state(&buffer);

    println!("\nDemo 4: True Color (24-bit RGB)");
    write_and_execute(
        &mut buffer,
        &mut parser,
        b"\x1b[38;2;255;128;0mOrange RGB\x1b[0m \x1b[38;2;128;0;255mPurple RGB\x1b[0m\n",
    );
    print_buffer_state(&buffer);

    println!("\nDemo 5: Scrollback Buffer");
    buffer.clear();
    for i in 0..15 {
        let line = format!("Line {} of 15\n", i + 1);
        write_and_execute(&mut buffer, &mut parser, line.as_bytes());
    }
    println!("  Buffer height: 10 rows");
    println!("  Lines written: 15");
    println!("  Scrollback: {} lines saved", buffer.scrollback_len());

    println!("\nDemo 6: Alternate Screen Mode");
    println!(
        "  Before enter: is_alternate = {}",
        buffer.is_alternate_screen()
    );
    write_and_execute(&mut buffer, &mut parser, b"\x1b[?1049h");
    println!(
        "  After enter:  is_alternate = {}",
        buffer.is_alternate_screen()
    );
    write_and_execute(&mut buffer, &mut parser, b"In alternate screen!");
    write_and_execute(&mut buffer, &mut parser, b"\x1b[?1049l");
    println!(
        "  After exit:   is_alternate = {}",
        buffer.is_alternate_screen()
    );

    println!("\nDemo 7: Dirty Region Tracking");
    buffer.clear();
    let grid = buffer.grid();
    println!("  After clear: has_dirty = {}", grid.has_dirty_rows());
    println!("  Row 0 dirty: {}", grid.is_row_dirty(0));
    println!("  Row 5 dirty: {}", grid.is_row_dirty(5));

    println!("\nDemo 8: Cursor Control");
    buffer.clear();
    write_and_execute(&mut buffer, &mut parser, b"\x1b[5;10H");
    let (x, y) = buffer.cursor_pos();
    println!("  ESC[5;10H -> cursor at ({}, {})", x, y);

    write_and_execute(&mut buffer, &mut parser, b"\x1b[2A");
    let (x, y) = buffer.cursor_pos();
    println!("  ESC[2A    -> cursor at ({}, {})", x, y);

    println!("\nAll demos completed!");
}

fn write_and_execute(buffer: &mut TerminalBuffer, parser: &mut AnsiParser, data: &[u8]) {
    let commands = parser.parse(data).unwrap();
    for cmd in commands {
        buffer.execute_command(cmd).unwrap();
    }
}

fn print_buffer_state(buffer: &TerminalBuffer) {
    let (x, y) = buffer.cursor_pos();
    println!("  Cursor position: ({}, {})", x, y);

    let grid = buffer.grid();
    for row_idx in 0..grid.height() {
        if let Some(row) = grid.row(row_idx) {
            let text: String = row.iter().map(|c| c.character).collect();
            let trimmed = text.trim_end();
            if !trimmed.is_empty() {
                println!("  Row {}: \"{}\"", row_idx, trimmed);
                break;
            }
        }
    }
}
