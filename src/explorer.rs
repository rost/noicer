use std::{io::Write, path::Path};

use std::process::Command;

use crossterm::{
    event::{self, Event},
    execute, queue, style,
    terminal::{self, ClearType},
};
use tempfile::NamedTempFile;

use crate::cursor::Cursor;
use crate::engine::{Engine, Mode, OpType};
use crate::file_cursor::FileCursor;
use crate::lines::Lines;
use crate::tar_cursor::TarCursor;

pub struct Config {
    pub editor: String,
    pub pager: String,
    pub shell: String,
}

impl Config {
    fn new() -> Config {
        Config {
            editor: String::from("vim"),
            pager: String::from("less"),
            shell: String::from("bash"),
        }
    }
}

pub struct State {
    pub config: Config,
    pub running: bool,
    pub tar: bool,
}

impl State {
    fn new() -> State {
        State {
            config: Config::new(),
            running: true,
            tar: false,
        }
    }
}

pub fn run<W>(w: &mut W) -> anyhow::Result<()>
where
    W: Write,
{
    execute!(w, terminal::EnterAlternateScreen)?;

    terminal::enable_raw_mode()?;

    let mut state = State::new();
    let mut file_cursor = FileCursor::new();
    let mut tar_cursor = TarCursor::new();
    let mut engine = Engine::new();

    let cwd = std::env::current_dir()?;

    file_cursor.init(&cwd)?;

    loop {
        if !state.running {
            break;
        }

        queue!(
            w,
            style::ResetColor,
            terminal::Clear(ClearType::All),
            crossterm::cursor::Hide,
            crossterm::cursor::MoveTo(1, 1)
        )?;

        let lines = match state.tar {
            true => {
                tar_cursor.init(&file_cursor.selected())?;
                Lines::new().format(&mut tar_cursor)?
            }
            false => Lines::new().format(&mut file_cursor)?,
        };

        for line in lines {
            queue!(w, style::Print(&line), crossterm::cursor::MoveToNextLine(1))?;
        }

        if engine.mode() == &Mode::Search {
            let (_, term_height) = terminal::size()?;
            queue!(
                w,
                crossterm::cursor::MoveTo(0, term_height),
                style::Print(format!("/{}", &engine.search_term()))
            )?;
        }

        w.flush()?;

        match state.tar {
            true => match handle_keypress(&mut tar_cursor, &mut engine) {
                Ok(res) => {
                    if let Some(op) = res {
                        let _res = run_op(&mut state, op, &mut tar_cursor, &mut engine)?;
                    }
                }
                Err(_) => {
                    break;
                }
            },
            false => match handle_keypress(&mut file_cursor, &mut engine) {
                Ok(res) => {
                    if let Some(op) = res {
                        let _res = run_op(&mut state, op, &mut file_cursor, &mut engine)?;
                    }
                }
                Err(_) => {
                    break;
                }
            },
        }
    }

    execute!(
        w,
        style::ResetColor,
        crossterm::cursor::Show,
        terminal::LeaveAlternateScreen
    )?;

    Ok(terminal::disable_raw_mode()?)
}

fn handle_keypress(cursor: &mut dyn Cursor, engine: &mut Engine) -> anyhow::Result<Option<OpType>> {
    let mut op = None;
    if let Event::Key(ke) = event::read()? {
        op = engine.push(ke)?;
        if engine.mode() == &Mode::Search && !engine.search_term().is_empty() {
            cursor.search(engine.search_term())?;
        }
    }
    if engine.mode() != &Mode::Search {
        engine.clear_search_term();
    } else {
        return Ok(None);
    }
    Ok(op)
}

fn run_op(
    state: &mut State,
    op: OpType,
    cursor: &mut dyn Cursor,
    engine: &mut Engine,
) -> anyhow::Result<bool> {
    match op {
        // simple
        OpType::Opq => state.running = false,
        OpType::OpG => cursor.move_bottom()?,
        OpType::Opj(n) => cursor.move_down(n)?,
        OpType::Opk(n) => cursor.move_up(n)?,
        OpType::Oph => {
            if state.tar
                && cursor.selected().parent().unwrap_or(Path::new("")) == cursor.start_dir()
            {
                state.tar = false
            } else {
                cursor.move_out()?
            }
        }
        OpType::Opl => {
            let selected = cursor.selected();
            if selected.is_dir() || selected.to_str().unwrap_or("").ends_with('/') {
                cursor.move_in()?
            } else if selected.extension().unwrap_or_default() == "tar" {
                state.tar = true;
            } else if state.tar && !selected.ends_with("..") {
                if let Some(tar_cursor) = cursor.as_any_mut().downcast_mut::<TarCursor>() {
                    let content = tar_cursor.read_file_content(&selected)?;
                    let mut temp_file = NamedTempFile::new()?;
                    temp_file.write_all(&content)?;
                    run_prog("bat", temp_file.path())?;
                }
            } else {
                run_prog("bat", &selected)?
            }
        }
        OpType::Opdot => cursor.toggle_hidden_files()?,
        OpType::Opcasing => cursor.toggle_case_sensitivity()?,
        OpType::Opsortdir => cursor.sort_dir()?,
        OpType::Opsortname => cursor.sort_name()?,
        OpType::Opsortsize => cursor.sort_size()?,
        OpType::Opsorttime => cursor.sort_time()?,
        OpType::Opslash => engine.toggle_search(),
        OpType::Oppage => run_prog(&state.config.pager, &cursor.selected())?,
        OpType::Opedit => run_prog(&state.config.editor, &cursor.selected())?,
        OpType::Opbang => run_prog(&state.config.shell, &cursor.current_dir())?,
        // complex
        OpType::Opgg => cursor.move_top()?,
        _ => return Ok(false),
    };
    Ok(true)
}

pub fn run_prog(prog: &str, path: &Path) -> anyhow::Result<()> {
    let mut out = match path.is_dir() {
        true => {
            std::env::set_current_dir(path)?;
            Command::new(prog)
                .spawn()
                .expect("pager command failed to start")
        }
        false => Command::new(prog)
            .arg(path)
            .spawn()
            .expect("pager command failed to start"),
    };
    out.wait().expect("failed while waiting");
    Ok(())
}
