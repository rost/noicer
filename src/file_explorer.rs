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

pub struct Config {
    editor: String,
    pager: String,
    shell: String,
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

struct State {
    config: Config,
    cursor: Cursor,
    line: String,
    running: bool,
    search: bool,
    search_term: String,
}

impl State {
    fn new() -> State {
        State {
            config: Config::new(),
            cursor: Cursor::new(),
            line: String::new(),
            running: true,
            search: false,
            search_term: String::new(),
        }
    }

    fn toggle_search(&mut self) {
        self.search = !self.search;
    }
}

pub fn run<W>(w: &mut W) -> anyhow::Result<()>
where
    W: Write,
{
    execute!(w, terminal::EnterAlternateScreen)?;

    terminal::enable_raw_mode()?;

    let mut state = State::new();

    state.cursor.init()?;

    loop {
        if !state.running {
            break;
        }

        queue!(
            w,
            style::ResetColor,
            terminal::Clear(ClearType::All),
            cursor::Hide,
            cursor::MoveTo(1, 1)
        )?;

        let screen_lines = format_lines(
            state.cursor.current_dir(),
            state.cursor.current_siblings()?,
            state.cursor.pos()?,
        )?;

        for line in screen_lines {
            queue!(w, style::Print(&line), cursor::MoveToNextLine(1))?;
        }

        if state.search {
            let (_, term_height) = terminal::size()?;
            queue!(
                w,
                cursor::MoveTo(0, term_height),
                style::Print(format!("/{}", &state.search_term))
            )?;
        }

        w.flush()?;

        match handle_keypress(&mut state) {
            Ok(_) => (),
            Err(_) => {
                break;
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

fn handle_keypress(state: &mut State) -> anyhow::Result<()> {
    if let Event::Key(KeyEvent { code, .. }) = event::read()? {
        match code {
            KeyCode::Char(c) => {
                if state.search {
                    state.search_term.push(c);
                } else {
                    state.line.push(c);
                }
            }
            KeyCode::Backspace => {
                if state.search {
                    if state.search_term.is_empty() {
                        state.toggle_search();
                        return Ok(());
                    }
                    state.search_term.pop();
                }
            }
            KeyCode::Esc => {
                if state.search {
                    state.toggle_search();
                }
            }
            KeyCode::Enter => {
                if state.search {
                    state.toggle_search();
                } else {
                    return Ok(());
                }
            }
            _ => {}
        }

        if state.search && !state.search_term.is_empty() {
            state.cursor.search(&state.search_term)?;
        }

        if state.search {
            return Ok(());
        }
    }

    if !state.search {
        state.search_term = String::new();

        let op = run_op(state)?;

        if op.is_some() || state.line.len() > 2 {
            state.line = String::new();
        }
    }
    Ok(())
}

fn run_op(state: &mut State) -> anyhow::Result<Option<Op>> {
    let mut op = parse_op(state.line.as_str());
    match &op {
        Some(o) => match &o.optype {
            // simple
            OpType::Opq => state.running = false,
            OpType::OpG => state.cursor.move_bottom()?,
            OpType::Opj => state.cursor.move_down(1)?,
            OpType::Opk => state.cursor.move_up(1)?,
            OpType::Oph => state.cursor.move_out()?,
            OpType::Opl => {
                if state.cursor.selected().is_dir() {
                    state.cursor.move_in()?
                } else {
                    run_prog("bat", &state.cursor.selected())?
                }
            }
            OpType::Opdot => state.cursor.toggle_hidden_files()?,
            OpType::Opcasing => state.cursor.toggle_case_sensitivity()?,
            OpType::Opsortdir => state.cursor.sort_dir()?,
            OpType::Opsortname => state.cursor.sort_name()?,
            OpType::Opsortsize => state.cursor.sort_size()?,
            OpType::Opsorttime => state.cursor.sort_time()?,
            OpType::Opslash => state.toggle_search(),
            OpType::Oppage => run_prog(&state.config.pager, &state.cursor.selected())?,
            OpType::Opedit => run_prog(&state.config.editor, &state.cursor.selected())?,
            OpType::Opbang => run_prog(&state.config.shell, &state.cursor.current_dir())?,
            // complex
            OpType::Opgg => state.cursor.move_top()?,
            OpType::Opnj => state.cursor.move_down(o.arg.parse::<i32>()?)?,
            OpType::Opnk => state.cursor.move_up(o.arg.parse::<i32>()?)?,
            _ => op = None,
        },
        None => op = None,
    }
    Ok(op)
}

fn run_prog(prog: &str, path: &Path) -> anyhow::Result<()> {
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

fn parse_op(line: &str) -> Option<Op> {
    if line.is_empty() {
        return None;
    }
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
            optype: parse_optype(&op, &arg),
            arg,
        }
    }
}

fn parse_optype(op: &str, arg: &str) -> OpType {
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
        "i" => OpType::Opcasing,
        "d" => OpType::Opsortdir,
        "n" => OpType::Opsortname,
        "s" => OpType::Opsortsize,
        "t" => OpType::Opsorttime,
        "/" => OpType::Opslash,
        "p" => OpType::Oppage,
        "e" => OpType::Opedit,
        "!" => OpType::Opbang,
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
    Opcasing,
    Opsortdir,
    Opsortname,
    Opsortsize,
    Opsorttime,
    Opslash,
    Oppage,
    Opedit,
    Opbang,
    None,
}
