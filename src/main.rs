use std::{
    collections::HashMap,
    io::{self, Write},
};

pub use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEvent},
    execute, queue, style,
    terminal::{self, ClearType},
    Command, Result,
};

pub struct Op {
    pub op_type: String,
    pub path: Option<String>,
}

impl Op {
    pub fn new(op_type: String, path: String) -> Op {
        Op {
            op_type,
            path: Some(path),
        }
    }
}

pub struct State {
    pub cursor: i32,
    pub dir: String,
    pub paths: HashMap<String, i32>,
    pub prev_op: Option<Op>,
    pub screen_lines: Vec<String>,
}

fn run<W>(w: &mut W) -> Result<()>
where
    W: Write,
{
    execute!(w, terminal::EnterAlternateScreen)?;

    terminal::enable_raw_mode()?;

    let mut state = State {
        cursor: 0,
        dir: std::env::current_dir()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string(),
        screen_lines: format_screen_lines(0),
        paths: HashMap::new(),
        prev_op: None,
    };

    loop {
        queue!(
            w,
            style::ResetColor,
            terminal::Clear(ClearType::All),
            cursor::Hide,
            cursor::MoveTo(1, 1)
        )?;

        let current_dir = std::env::current_dir().unwrap();
        state.dir = current_dir.to_str().unwrap().to_string();

        state.cursor = if state.paths.contains_key(state.dir.as_str()) {
            state.paths.get(state.dir.as_str()).unwrap().clone()
        } else {
            match &state.prev_op {
                Some(op) if op.op_type == "out" => 0,
                Some(_) => 0,
                None => 0,
            }
        };

        state.screen_lines = format_screen_lines(state.cursor);

        for line in &state.screen_lines {
            queue!(w, style::Print(line), cursor::MoveToNextLine(1))?;
        }

        w.flush()?;

        match read_char()? {
            'j' => {
                state.cursor = move_down(&state)?;
                state.prev_op = None;
            }
            'k' => {
                state.cursor = move_up(&state)?;
                state.prev_op = None;
            }
            'h' => {
                let (cursor, op) = move_out_of_dir(&state)?;
                state.cursor = cursor;
                state.prev_op = op
            }
            'l' => {
                state.cursor = move_into_dir(&state)?;
                state.prev_op = None;
            }
            'q' => break,
            _ => (),
        };

        state.paths.insert(state.dir.to_string(), state.cursor);
    }

    execute!(
        w,
        style::ResetColor,
        cursor::Show,
        terminal::LeaveAlternateScreen
    )?;

    terminal::disable_raw_mode()
}

fn move_down(state: &State) -> Result<i32> {
    let cursor = if state.cursor + 1 < (state.screen_lines.len() - 2) as i32 {
        state.cursor + 1
    } else {
        state.cursor
    };
    Ok(cursor)
}

fn move_up(state: &State) -> Result<i32> {
    let cursor = if state.cursor - 1 >= 0 {
        state.cursor - 1
    } else {
        0
    };
    Ok(cursor)
}

fn move_out_of_dir(state: &State) -> Result<(i32, Option<Op>)> {
    let op = Some(Op::new(String::from("out"), String::from(&state.dir)));
    std::env::set_current_dir("..")?;
    Ok((state.cursor, op))
}

fn move_into_dir(state: &State) -> Result<i32> {
    let path = state.screen_lines[(state.cursor + 2) as usize].trim_start();
    let newdir = path.trim_end_matches('/');
    let newdir = str::replace(&newdir, ">", " ");
    let newdir = newdir.trim_start();
    let current_dir = std::env::current_dir().unwrap();
    let newdir = current_dir.join(newdir);
    if path.ends_with('/') {
        std::env::set_current_dir(newdir).unwrap();
    }
    Ok(state.cursor)
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

fn format_screen_lines(cursor: i32) -> Vec<String> {
    let cursor = cursor + 2;
    let mut lines = Vec::new();
    let current_dir = std::env::current_dir().unwrap();
    lines.push(format!("{}", current_dir.display()));
    lines.push(String::from(""));

    let mut entries = Vec::new();
    for entry in std::fs::read_dir(".").unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        let dir = path.file_name().unwrap().to_str().unwrap();
        if path.is_dir() {
            entries.push(format!("   {}/", dir));
        } else {
            entries.push(format!("   {}", dir));
        }
    }

    entries.sort();

    for entry in entries {
        lines.push(entry);
    }

    lines[cursor as usize] = format!(" > {}", lines[cursor as usize].trim_start());

    lines
}
