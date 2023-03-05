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

    let mut line = String::new();

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
            if let Event::Key(KeyEvent { code, .. }) = event::read()? {
                match code {
                    KeyCode::Enter => {
                        continue;
                    }
                    KeyCode::Char(c) => {
                        line.push(c);
                    }
                    _ => {}
                }
            }

            if let "q" = line.as_str() {
                break;
            };

            let simple_op: bool = match line.as_str() {
                "G" => {
                    cursor.move_bottom()?;
                    true
                }
                "j" => {
                    cursor.move_down(1)?;
                    true
                }
                "k" => {
                    cursor.move_up(1)?;
                    true
                }
                "h" => {
                    cursor.move_out()?;
                    true
                }
                "l" => {
                    cursor.move_in()?;
                    true
                }
                "." => {
                    cursor.toggle_hidden_files()?;
                    true
                }
                "/" => {
                    search = toggle_search(search);
                    true
                }
                _ => false,
            };

            let mut complex_op: bool = false;
            if line.len() > 1 {
                complex_op = match line.as_str() {
                    "gg" => {
                        cursor.move_top()?;
                        true
                    }
                    cmd => {
                        let (arg, op) = cmd.split_at(1);
                        let arg = arg.parse::<i32>().unwrap_or(1);
                        match op {
                            "j" => {
                                cursor.move_down(arg)?;
                                true
                            }
                            "k" => {
                                cursor.move_up(arg)?;
                                true
                            }
                            _ => false,
                        };
                        true
                    }
                };
            }

            if simple_op || complex_op {
                line = String::new();
            }
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
