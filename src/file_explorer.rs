use std::{
    io::Write,
    path::{Path, PathBuf},
};

use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEvent},
    execute, queue, style,
    terminal::{self, ClearType},
    Result,
};

use crate::cursor::Cursor;

pub fn run<W>(w: &mut W) -> anyhow::Result<()>
where
    W: Write,
{
    execute!(w, terminal::EnterAlternateScreen)?;

    terminal::enable_raw_mode()?;

    let mut cursor = Cursor::new(std::env::current_dir()?);
    cursor.init()?;

    loop {
        queue!(
            w,
            style::ResetColor,
            terminal::Clear(ClearType::All),
            cursor::Hide,
            cursor::MoveTo(1, 1)
        )?;

        let screen_lines = format_lines(&cursor)?;
        for line in screen_lines {
            queue!(w, style::Print(line), cursor::MoveToNextLine(1))?;
        }

        w.flush()?;

        match read_char()? {
            'q' => break,
            char => handle_keypress(&char, &mut cursor)?,
        };
    }

    execute!(
        w,
        style::ResetColor,
        cursor::Show,
        terminal::LeaveAlternateScreen
    )?;

    Ok(terminal::disable_raw_mode()?)
}

fn format_lines(cursor: &Cursor) -> Result<Vec<String>> {
    let empty_dir = vec![PathBuf::from("   ../")];
    let content = if !cursor.siblings()?.is_empty() {
        cursor.siblings()?
    } else {
        empty_dir
    };

    let mut lines = Vec::new();
    lines.push(format!("{}", cursor.current_dir().display()));
    lines.push(String::from(""));

    for entry in content {
        lines.push(format_pathbuf(&entry)?);
    }

    let index = (cursor.pos()? + 2) as usize;
    lines[index] = format!(" > {}", lines[index].trim_start());

    Ok(lines)
}

fn format_pathbuf(path: &Path) -> Result<String> {
    let f = path.file_name();
    let s = match f {
        Some(v) => v.to_str(),
        None => None,
    };
    let r = match (path.is_dir(), s) {
        (true, Some(v)) => format!("   {v}/"),
        (false, Some(v)) => format!("   {v}"),
        _ => String::from(""),
    };
    Ok(r)
}

fn read_char() -> Result<char> {
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

fn handle_keypress(char: &char, arrow: &mut Cursor) -> Result<()> {
    match char {
        'j' => arrow.move_down()?,
        'k' => arrow.move_up()?,
        'h' => arrow.move_out()?,
        'l' => arrow.move_in()?,
        _ => (),
    };
    Ok(())
}
