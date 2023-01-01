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

    loop {
        queue!(
            w,
            style::ResetColor,
            terminal::Clear(ClearType::All),
            cursor::Hide,
            cursor::MoveTo(1, 1)
        )?;

        for line in screen_lines() {
            queue!(w, style::Print(line), cursor::MoveToNextLine(1))?;
        }

        w.flush()?;

        match read_char()? {
            // '1' => test::cursor::run(w)?,
            // '2' => test::color::run(w)?,
            // '3' => test::attribute::run(w)?,
            // '4' => test::event::run(w)?,
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

fn screen_lines() -> Vec<String> {
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
    lines
}
