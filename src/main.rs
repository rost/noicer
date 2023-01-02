use std::io::{self, Write};

pub use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEvent},
    execute, queue, style,
    terminal::{self, ClearType},
    Command, Result,
};

pub struct State {
    pub cursor: i32,
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
        screen_lines: get_screen_lines(0),
    };

    loop {
        queue!(
            w,
            style::ResetColor,
            terminal::Clear(ClearType::All),
            cursor::Hide,
            cursor::MoveTo(1, 1)
        )?;

        state = State {
            cursor: state.cursor,
            screen_lines: get_screen_lines(state.cursor),
        };

        for line in &state.screen_lines {
            queue!(w, style::Print(line), cursor::MoveToNextLine(1))?;
        }

        w.flush()?;

        match read_char()? {
            'j' => {
                if state.cursor + 1 < (state.screen_lines.len() - 2) as i32 {
                    state.cursor += 1;
                }
            }
            'k' => {
                if state.cursor - 1 >= 0 {
                    state.cursor -= 1;
                }
            }
            'h' => {
                std::env::set_current_dir("..")?;
                state.cursor = 0;
            }
            'l' => {
                state.cursor = move_into_dir(&state)?;
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
    Ok(0)
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

fn get_screen_lines(cursor: i32) -> Vec<String> {
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
