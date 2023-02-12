use std::{
    collections::HashMap,
    ffi::OsStr,
    io::{self, Write},
    path::{Path, PathBuf},
};

use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEvent},
    execute, queue, style,
    terminal::{self, ClearType},
    Result,
};

#[derive(PartialEq)]
enum OpKind {
    Out,
}

struct Op {
    kind: OpKind,
    path: PathBuf,
}

impl Op {
    fn new(kind: OpKind, path: PathBuf) -> Op {
        Op { kind, path }
    }
}

struct Cursor {
    dir: PathBuf,
    paths: HashMap<PathBuf, i32>,
    point: i32,
    prev_op: Option<Op>,
    siblings: Vec<PathBuf>,
}

impl Cursor {
    fn new() -> Result<Cursor> {
        Ok(Cursor {
            dir: std::env::current_dir()?,
            paths: HashMap::new(),
            point: 0,
            prev_op: None,
            siblings: Vec::new(),
        })
    }

    fn pos(&self) -> i32 {
        self.point
    }

    fn update_dir(&mut self) -> Result<()> {
        self.dir = std::env::current_dir()?;
        Ok(())
    }

    fn update_dir_content(&mut self) -> Result<()> {
        let mut entries = Vec::new();
        for entry in std::fs::read_dir(".")? {
            entries.push(entry?.path());
        }
        entries.sort();
        self.siblings = entries;
        Ok(())
    }

    fn update_pos(&mut self) -> Result<()> {
        let path_pos = self.paths.get(&self.dir);
        let prev = self.prev_op.as_ref().map(|op| op.kind == OpKind::Out);
        let last = self.prev_op.as_ref().and_then(|op| op.path.file_name());
        let pos = match (path_pos, prev, last) {
            (Some(&cursor), _, _) => cursor,
            (None, Some(true), Some(last)) => {
                let index = self
                    .siblings
                    .iter()
                    .position(|p| p.file_name() == Some(last))
                    .unwrap_or(0);
                index as i32
            }
            _ => 0,
        };
        self.point = pos;
        Ok(())
    }

    fn move_down(&mut self) -> Result<()> {
        if self.point + 1 < self.siblings.len() as i32 {
            self.point += 1;
        }
        self.prev_op = None;
        Ok(())
    }

    fn move_up(&mut self) -> Result<()> {
        if self.point > 0 {
            self.point -= 1;
        } else {
            self.point = 0;
        }
        self.prev_op = None;
        Ok(())
    }

    fn move_out_of_dir(&mut self) -> Result<()> {
        std::env::set_current_dir("..")?;
        let op = Some(Op::new(OpKind::Out, self.dir.clone()));
        self.point = self.pos();
        self.prev_op = op;
        Ok(())
    }

    fn move_into_dir(&mut self) -> Result<()> {
        if !self.siblings.is_empty() {
            let path = &self.siblings[(self.pos()) as usize];
            let file = path.file_name().unwrap_or(OsStr::new(""));
            let newdir = self.dir.join(file);
            if newdir.is_dir() {
                std::env::set_current_dir(newdir)?;
            }
        }
        self.point = self.pos();
        self.prev_op = None;
        Ok(())
    }

    fn before(&mut self) -> Result<()> {
        self.update_dir()?;
        self.update_dir_content()?;
        self.update_pos()?;
        Ok(())
    }

    fn after(&mut self) -> Result<()> {
        self.paths.insert(self.dir.clone(), self.pos());
        Ok(())
    }
}

fn main() -> anyhow::Result<()> {
    let mut stdout = io::stdout();
    run(&mut stdout)?;
    Ok(())
}

fn run<W>(w: &mut W) -> anyhow::Result<()>
where
    W: Write,
{
    execute!(w, terminal::EnterAlternateScreen)?;

    terminal::enable_raw_mode()?;

    let mut cursor = Cursor::new()?;

    loop {
        queue!(
            w,
            style::ResetColor,
            terminal::Clear(ClearType::All),
            cursor::Hide,
            cursor::MoveTo(1, 1)
        )?;

        cursor.before()?;

        let screen_lines = format_lines(&cursor)?;
        for line in screen_lines {
            queue!(w, style::Print(line), cursor::MoveToNextLine(1))?;
        }

        w.flush()?;

        match read_char()? {
            'q' => break,
            char => handle_keypress(&char, &mut cursor)?,
        };

        cursor.after()?;
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
    let content = if !cursor.siblings.is_empty() {
        &cursor.siblings
    } else {
        &empty_dir
    };

    let mut lines = Vec::new();
    lines.push(format!("{}", cursor.dir.display()));
    lines.push(String::from(""));

    for entry in content {
        lines.push(format_pathbuf(entry)?);
    }

    let index = (cursor.pos() + 2) as usize;
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
        'h' => arrow.move_out_of_dir()?,
        'l' => arrow.move_into_dir()?,
        _ => (),
    };
    Ok(())
}
