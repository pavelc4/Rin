//! Demo binary to test Rin terminal emulator features
//!
//! Run with: cargo run --example demo

use rin::{AnsiParser, Command, TerminalBuffer};

fn main() {
    println!("=== Rin Terminal Engine Demo ===\n");

    // Create a small terminal buffer (40x10)
    let mut buffer = TerminalBuffer::new(40, 10);
    let mut parser = AnsiParser::new();

    // Demo 1: Basic text
    println!("📝 Demo 1: Basic Text");
    write_and_execute(&mut buffer, &mut parser, b"Hello, Rin Terminal!\n");
    print_buffer_state(&buffer);

    // Demo 2: Colors (basic ANSI)
    println!("\n🎨 Demo 2: Basic ANSI Colors");
    write_and_execute(
        &mut buffer,
        &mut parser,
        b"\x1b[31mRed\x1b[0m \x1b[32mGreen\x1b[0m \x1b[34mBlue\x1b[0m\n",
    );
    print_buffer_state(&buffer);

    // Demo 3: 256 Color
    println!("\n🌈 Demo 3: 256 Color Palette");
    write_and_execute(
        &mut buffer,
        &mut parser,
        b"\x1b[38;5;196mRed256\x1b[0m \x1b[38;5;46mGreen256\x1b[0m\n",
    );
    print_buffer_state(&buffer);

    // Demo 4: True Color (RGB)
    println!("\n✨ Demo 4: True Color (24-bit RGB)");
    write_and_execute(
        &mut buffer,
        &mut parser,
        b"\x1b[38;2;255;128;0mOrange RGB\x1b[0m \x1b[38;2;128;0;255mPurple RGB\x1b[0m\n",
    );
    print_buffer_state(&buffer);

    // Demo 5: Scrollback
    println!("\n📜 Demo 5: Scrollback Buffer");
    buffer.clear();
    for i in 0..15 {
        let line = format!("Line {} of 15\n", i + 1);
        write_and_execute(&mut buffer, &mut parser, line.as_bytes());
    }
    println!("  Buffer height: 10 rows");
    println!("  Lines written: 15");
    println!("  Scrollback: {} lines saved", buffer.scrollback_len());

    // Demo 6: Alternate Screen
    println!("\n🖥️  Demo 6: Alternate Screen Mode");
    println!(
        "  Before enter: is_alternate = {}",
        buffer.is_alternate_screen()
    );

    // Enter alternate screen
    write_and_execute(&mut buffer, &mut parser, b"\x1b[?1049h");
    println!(
        "  After enter:  is_alternate = {}",
        buffer.is_alternate_screen()
    );

    // Write something in alternate
    write_and_execute(&mut buffer, &mut parser, b"In alternate screen!");

    // Exit alternate screen
    write_and_execute(&mut buffer, &mut parser, b"\x1b[?1049l");
    println!(
        "  After exit:   is_alternate = {}",
        buffer.is_alternate_screen()
    );

    // Demo 7: Dirty Tracking
    println!("\n🔄 Demo 7: Dirty Region Tracking");
    buffer.clear();
    let grid = buffer.grid();
    println!("  After clear: has_dirty = {}", grid.has_dirty_rows());
    println!("  Row 0 dirty: {}", grid.is_row_dirty(0));
    println!("  Row 5 dirty: {}", grid.is_row_dirty(5));

    // Demo 8: Cursor Movement
    println!("\n🎯 Demo 8: Cursor Control");
    buffer.clear();
    write_and_execute(&mut buffer, &mut parser, b"\x1b[5;10H"); // Move to row 5, col 10
    let (x, y) = buffer.cursor_pos();
    println!("  ESC[5;10H -> cursor at ({}, {})", x, y);

    write_and_execute(&mut buffer, &mut parser, b"\x1b[2A"); // Move up 2
    let (x, y) = buffer.cursor_pos();
    println!("  ESC[2A    -> cursor at ({}, {})", x, y);

    println!("\n✅ All demos completed!");
    println!("\n💡 Tip: Check src/tests.rs for unit tests covering all features.");
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

    // Print first visible line (non-empty)
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
