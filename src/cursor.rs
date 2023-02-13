use std::{collections::HashMap, ffi::OsStr, path::PathBuf};

use crossterm::Result;

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

pub struct Cursor {
    dir: PathBuf,
    paths: HashMap<PathBuf, i32>,
    point: i32,
    prev_op: Option<Op>,
}

impl Cursor {
    pub fn new() -> Result<Cursor> {
        Ok(Cursor {
            dir: std::env::current_dir()?,
            paths: HashMap::new(),
            point: 0,
            prev_op: None,
        })
    }

    pub fn pos(&self) -> i32 {
        self.point
    }

    pub fn current_dir(&self) -> &PathBuf {
        &self.dir
    }

    pub fn siblings(&self) -> Result<Vec<PathBuf>> {
        let mut siblings = Vec::new();
        for entry in std::fs::read_dir(".")? {
            siblings.push(entry?.path());
        }
        siblings.sort();
        Ok(siblings)
    }

    pub fn update_dir(&mut self) -> Result<()> {
        self.dir = std::env::current_dir()?;
        Ok(())
    }

    pub fn update_pos(&mut self) -> Result<()> {
        let path_pos = self.paths.get(&self.dir);
        let prev = self.prev_op.as_ref().map(|op| op.kind == OpKind::Out);
        let last = self.prev_op.as_ref().and_then(|op| op.path.file_name());
        let pos = match (path_pos, prev, last) {
            (Some(&cursor), _, _) => cursor,
            (None, Some(true), Some(last)) => {
                let index = self
                    .siblings()?
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

    pub fn move_down(&mut self) -> Result<()> {
        if self.point + 1 < self.siblings()?.len() as i32 {
            self.point += 1;
        }
        self.prev_op = None;
        Ok(())
    }

    pub fn move_up(&mut self) -> Result<()> {
        if self.point > 0 {
            self.point -= 1;
        } else {
            self.point = 0;
        }
        self.prev_op = None;
        Ok(())
    }

    pub fn move_out_of_dir(&mut self) -> Result<()> {
        std::env::set_current_dir("..")?;
        let op = Some(Op::new(OpKind::Out, self.dir.clone()));
        self.point = self.pos();
        self.prev_op = op;
        Ok(())
    }

    pub fn move_into_dir(&mut self) -> Result<()> {
        if !self.siblings()?.is_empty() {
            let path = &self.siblings()?[(self.pos()) as usize];
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

    pub fn before(&mut self) -> Result<()> {
        self.update_dir()?;
        self.update_pos()?;
        Ok(())
    }

    pub fn after(&mut self) -> Result<()> {
        self.paths.insert(self.dir.clone(), self.pos());
        Ok(())
    }
}
