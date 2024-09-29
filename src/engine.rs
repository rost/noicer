use crossterm::event::{KeyCode, KeyEvent};

pub struct Engine {
    buffer: String,
    mode: Mode,
    search_term: String,
}

#[derive(PartialEq)]
pub enum Mode {
    Normal,
    Search,
}

impl Engine {
    pub fn new() -> Self {
        Engine {
            buffer: String::new(),
            mode: Mode::Normal,
            search_term: String::new(),
        }
    }

    pub fn push(&mut self, ke: KeyEvent) -> anyhow::Result<Option<OpType>> {
        match ke.code {
            KeyCode::Char(c) => {
                if self.mode == Mode::Normal {
                    let op = self.handle_char(c)?;
                    Ok(op)
                } else if self.mode == Mode::Search {
                    self.search_term.push(c);
                    Ok(None)
                } else {
                    Ok(None)
                }
            }

            KeyCode::Backspace => {
                if self.mode == Mode::Search {
                    if !self.search_term.is_empty() {
                        self.search_term.pop();
                    } else {
                        self.clear_search_term();
                        self.toggle_search();
                    }
                }
                Ok(None)
            }

            KeyCode::Esc => {
                if self.mode == Mode::Search {
                    self.clear_search_term();
                    self.toggle_search();
                    Ok(Some(OpType::Opabort))
                } else {
                    Ok(None)
                }
            }

            KeyCode::Enter => {
                if self.mode == Mode::Search {
                    self.clear_search_term();
                    self.toggle_search();
                    Ok(Some(OpType::Opabort))
                } else {
                    Ok(None)
                }
            }

            _ => Ok(None),
        }
    }

    fn handle_char(&mut self, c: char) -> anyhow::Result<Option<OpType>> {
        self.buffer.push(c);
        let op = self.buffer.pop();
        let mut res = None;
        if let Some(op) = op {
            res = self.parse_op(&op.to_string())?;
        }
        if res != Some(OpType::None) || self.buffer.len() > 2 {
            self.buffer = String::new();
        }
        if res == Some(OpType::None) {
            self.buffer
                .push(op.ok_or(anyhow::anyhow!("shouldn't happen"))?);
        }
        Ok(res)
    }

    fn parse_op(&mut self, op: &str) -> anyhow::Result<Option<OpType>> {
        let op = match op {
            // simple
            "q" => OpType::Opq,
            "G" => OpType::OpG,
            "j" => {
                if !self.buffer.is_empty() {
                    let n = self.buffer.parse::<i32>().unwrap_or(1);
                    OpType::Opj(n)
                } else {
                    OpType::Opj(1)
                }
            }
            "k" => {
                if !self.buffer.is_empty() {
                    let n = self.buffer.parse::<i32>().unwrap_or(1);
                    OpType::Opk(n)
                } else {
                    OpType::Opk(1)
                }
            }

            "h" => OpType::Oph,
            "l" => OpType::Opl,

            "." => OpType::Opdot,
            "~" => OpType::Opcasing,
            "d" => OpType::Opsortdir,
            "n" => OpType::Opsortname,
            "s" => OpType::Opsortsize,
            "t" => OpType::Opsorttime,
            "/" => OpType::Opslash,
            "p" => OpType::Oppage,
            "e" => OpType::Opedit,
            "!" => OpType::Opbang,
            "?" => OpType::Opquestion,

            // complex
            "g" => {
                if !self.buffer.is_empty() {
                    let o = self.buffer.pop().unwrap_or_default().to_string();

                    if o == "g" {
                        OpType::Opgg
                    } else {
                        OpType::None
                    }
                } else {
                    OpType::None
                }
            }

            _ => OpType::None,
        };
        Ok(Some(op))
    }

    pub fn toggle_search(&mut self) {
        match self.mode {
            Mode::Normal => {
                self.mode = Mode::Search;
            }
            Mode::Search => {
                self.mode = Mode::Normal;
            }
        }
    }

    pub fn mode(&self) -> &Mode {
        &self.mode
    }

    pub fn is_search(&self) -> bool {
        match self.mode {
            Mode::Normal => false,
            Mode::Search => true,
        }
    }

    pub fn search_term(&self) -> &str {
        &self.search_term
    }

    pub fn clear_search_term(&mut self) {
        self.search_term = String::new();
    }
}

impl Default for Engine {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(PartialEq)]
pub enum OpType {
    Opq,
    OpG,
    Opgg,
    Opj(i32),
    Opk(i32),
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
    Opabort,
    Opquestion,
    None,
}
