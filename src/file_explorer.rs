use std::{
    io::Write, path::Path
};

use std::process::Command;

use crossterm::{
    event::{self, Event}, execute, queue, style, terminal::{self, ClearType}
};

use crate::{cursor::Cursor, engine::{Mode, OpType}};
use crate::engine::Engine;
use crate::lines::Lines;

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
}

impl State {
    fn new() -> State {
        State {
            config: Config::new(),
            running: true,
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
    let mut cursor = Cursor::new();
    let mut engine = Engine::new();

    cursor.init()?;

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

        let lines = Lines::new().format(&cursor)?;

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

        match handle_keypress(&mut cursor, &mut engine) {
            Ok(res) => {
                if let Some(op) = res {
                    let _res = run_op(&mut state, op, &mut cursor, &mut engine)?;
                }
            }
            Err(_) => {
                break;
            }
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

fn handle_keypress(cursor: &mut Cursor, engine: &mut Engine) -> anyhow::Result<Option<OpType>> {
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
        return Ok(None)
    }
    Ok(op)
}

fn run_op(state: &mut State, op: OpType, cursor: &mut Cursor, engine: &mut Engine) -> anyhow::Result<bool> {
    match op {
        // simple
        OpType::Opq => state.running = false,
        OpType::OpG => cursor.move_bottom()?,
        OpType::Opj(n) => cursor.move_down(n)?,
        OpType::Opk(n) => cursor.move_up(n)?,
        OpType::Oph => cursor.move_out()?,
        OpType::Opl => {
            if cursor.selected().is_dir() {
                cursor.move_in()?
            } else {
                run_prog("bat", &cursor.selected())?
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
