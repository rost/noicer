use std::{
    io::Write,
    path::{Path, PathBuf},
};

use std::process::Command;

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

            let mut simple_op = parse_line(line.as_str());
            match &simple_op {
                Some(o) => match &o.optype {
                    OpType::Opq => break,
                    OpType::OpG => cursor.move_bottom()?,
                    OpType::Opj => cursor.move_down(1)?,
                    OpType::Opk => cursor.move_up(1)?,
                    OpType::Oph => cursor.move_out()?,
                    OpType::Opl => {
                        if cursor.selected().is_dir() {
                            cursor.move_in()?
                        } else {
                            run_pager(&cursor.selected())?
                        }
                    }
                    OpType::Opdot => cursor.toggle_hidden_files()?,
                    OpType::Opslash => search = toggle_search(search),
                    _ => simple_op = None,
                },
                None => simple_op = None,
            }

            let mut complex_op = None;
            if line.len() > 1 {
                complex_op = parse_line(line.as_str());
                match &complex_op {
                    Some(o) => match &o.optype {
                        OpType::Opgg => cursor.move_top()?,
                        OpType::Opnj => cursor.move_down(o.arg.parse::<i32>()?)?,
                        OpType::Opnk => cursor.move_up(o.arg.parse::<i32>()?)?,
                        _ => complex_op = None,
                    },
                    None => complex_op = None,
                }
            }

            if simple_op.is_some() || complex_op.is_some() || line.len() > 2 {
                line = String::new()
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

fn run_pager(path: &Path) -> anyhow::Result<()> {
    let mut out = Command::new("bat")
        .arg(path)
        .spawn()
        .expect("pager command failed to start");
    out.wait().expect("failed while waiting");
    Ok(())
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

fn parse_line(line: &str) -> Option<Op> {
    let (arg, op) = line.split_at(line.len() - 1);
    let op = Op::new(String::from(op), String::from(arg));
    Some(op)
}

struct Op {
    optype: OpType,
    arg: String,
}

impl Op {
    fn new(op: String, arg: String) -> Self {
        Self {
            optype: parse_op(&op, &arg),
            arg,
        }
    }
}

fn parse_op(op: &str, arg: &str) -> OpType {
    match op {
        "q" => OpType::Opq,
        "G" => OpType::OpG,
        "g" => {
            if arg == "g" {
                OpType::Opgg
            } else {
                OpType::None
            }
        }
        "j" => match arg {
            "" => OpType::Opj,
            _ => {
                let n = arg.parse::<i32>().unwrap_or(-1);
                if n >= 0 {
                    OpType::Opnj
                } else {
                    OpType::None
                }
            }
        },
        "k" => match arg {
            "" => OpType::Opk,
            _ => {
                let n = arg.parse::<i32>().unwrap_or(-1);
                if n >= 0 {
                    OpType::Opnk
                } else {
                    OpType::None
                }
            }
        },
        "h" => OpType::Oph,
        "l" => OpType::Opl,
        "." => OpType::Opdot,
        "/" => OpType::Opslash,
        _ => OpType::None,
    }
}

enum OpType {
    Opq,
    OpG,
    Opgg,
    Opnj,
    Opj,
    Opnk,
    Opk,
    Oph,
    Opl,
    Opdot,
    Opslash,
    None,
}
