use std::io::{self, Write};

pub use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEvent},
    execute, queue, style,
    terminal::{self, ClearType},
    Command, Result,
};

fn run<W>(w: &mut W) -> Result<()>
where
    W: Write,
{
    execute!(w, terminal::EnterAlternateScreen)?;

    terminal::enable_raw_mode()?;

    let mut cursor = 0;

    loop {
        queue!(
            w,
            style::ResetColor,
            terminal::Clear(ClearType::All),
            cursor::Hide,
            cursor::MoveTo(1, 1)
        )?;

        let screen_lines = screen_lines(cursor);

        for line in &screen_lines {
            queue!(w, style::Print(line), cursor::MoveToNextLine(1))?;
        }

        w.flush()?;

        match read_char()? {
            'j' => {
                if cursor + 1 < (screen_lines.len() - 2) as i32 {
                    cursor += 1;
                }
            }
            'k' => {
                if cursor - 1 >= 0 {
                    cursor -= 1;
                }
            }
            'q' => break,
            _ => {}
        };
    }

    execute!(
        w,
        style::ResetColor,
        cursor::Show,
        terminal::LeaveAlternateScreen
    )?;

    terminal::disable_raw_mode()
}

pub fn read_char() -> Result<char> {
    loop {
        if let Ok(Event::Key(KeyEvent {
            code: KeyCode::Char(c),
            ..
        })) = event::read()
        {
            return Ok(c);
        }
    }
}

pub fn buffer_size() -> Result<(u16, u16)> {
    terminal::size()
}

fn main() -> Result<()> {
    let mut stdout = io::stdout();
    run(&mut stdout)
}

fn screen_lines(cursor: i32) -> Vec<String> {
    let cursor = cursor + 2;
    let mut lines = Vec::new();
    let current_dir = std::env::current_dir().unwrap();
    lines.push(format!("{}", current_dir.display()));
    lines.push(String::from(""));
    for entry in std::fs::read_dir(".").unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        let dir = path.file_name().unwrap().to_str().unwrap();
        if path.is_dir() {
            lines.push(format!("   {}/", dir));
        } else {
            lines.push(format!("   {}", dir));
        }
    }

    lines[cursor as usize] = format!(" > {}", lines[cursor as usize].trim_start());

    lines
}
