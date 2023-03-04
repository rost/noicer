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

    let mut search = false;
    let mut search_term = String::new();

    let mut cursor = Cursor::new();
    cursor.init()?;

    loop {
        queue!(
            w,
            style::ResetColor,
            terminal::Clear(ClearType::All),
            cursor::Hide,
            cursor::MoveTo(1, 1)
        )?;

        let screen_lines = format_lines(
            cursor.current_dir(),
            cursor.current_siblings()?,
            cursor.pos()?,
        )?;
        for line in screen_lines {
            queue!(w, style::Print(line), cursor::MoveToNextLine(1))?;
        }

        if search {
            let (_, term_height) = terminal::size()?;
            queue!(
                w,
                cursor::MoveTo(0, term_height),
                style::Print(format!("/{}", search_term))
            )?;
        }

        w.flush()?;

        if search {
            if let Event::Key(KeyEvent { code, .. }) = event::read()? {
                match code {
                    KeyCode::Char(c) => {
                        search_term.push(c);
                    }
                    KeyCode::Backspace => {
                        if search_term.is_empty() {
                            search = toggle_search(search);
                            continue;
                        }
                        search_term.pop();
                    }
                    KeyCode::Esc | KeyCode::Enter => {
                        search = toggle_search(search);
                    }
                    _ => {}
                }
                if !search_term.is_empty() {
                    cursor.search(&search_term)?
                }
            }
        } else {
            search_term = String::new();
            match read_char()? {
                'q' => break,
                '/' => search = toggle_search(search),
                char => handle_keypress(&char, &mut cursor)?,
            };
        }
    }

    execute!(
        w,
        style::ResetColor,
        cursor::Show,
        terminal::LeaveAlternateScreen
    )?;

    Ok(terminal::disable_raw_mode()?)
}

fn toggle_search(search: bool) -> bool {
    !search
}

fn format_lines(
    current_dir: PathBuf,
    current_siblings: Vec<PathBuf>,
    pos: i32,
) -> Result<Vec<String>> {
    let content = if !current_siblings.is_empty() {
        current_siblings
    } else {
        vec![PathBuf::from("   ../")]
    };

    let mut lines = Vec::new();
    lines.push(format!("{}", current_dir.display()));
    lines.push(String::from(""));

    for entry in content {
        lines.push(format_pathbuf(&entry)?);
    }

    let index = (pos + 2) as usize;
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
        'G' => arrow.move_bottom()?,
        'g' => arrow.move_top()?,
        '.' => arrow.toggle_hidden_files()?,
        _ => (),
    };
    Ok(())
}
