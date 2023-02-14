use std::{collections::HashMap, ffi::OsStr, path::PathBuf};

use crossterm::Result;

pub struct Cursor {
    cwd: PathBuf,
    paths: HashMap<PathBuf, PathBuf>,
    selected: PathBuf,
}

impl Cursor {
    pub fn new(cwd: PathBuf) -> Cursor {
        Cursor {
            cwd,
            paths: HashMap::new(),
            selected: PathBuf::new(),
        }
    }

    pub fn init(&mut self) -> Result<()> {
        self.cwd = std::env::current_dir()?;
        self.paths = HashMap::new();
        self.selected = self.first_child()?;
        Ok(())
    }

    pub fn pos(&self) -> Result<i32> {
        let siblings = self.siblings()?;
        let pos = siblings
            .iter()
            .position(|p| p == &self.selected)
            .unwrap_or(0) as i32;
        Ok(pos)
    }

    pub fn current_dir(&self) -> &PathBuf {
        &self.cwd
    }

    pub fn siblings(&self) -> Result<Vec<PathBuf>> {
        let mut siblings = Vec::new();
        for entry in std::fs::read_dir(".")? {
            if let Some(f) = entry?.path().file_name() {
                siblings.push(self.cwd.join(f))
            }
        }
        siblings.sort();
        Ok(siblings)
    }

    fn first_child(&self) -> Result<PathBuf> {
        let mut children = Vec::new();
        for entry in std::fs::read_dir(&self.cwd)? {
            let child = entry?.path();
            children.push(child);
        }
        children.sort();
        if children.is_empty() {
            Ok(self.cwd.clone())
        } else {
            Ok(children[0].clone())
        }
    }

    pub fn move_down(&mut self) -> Result<()> {
        let siblings = self.siblings()?;
        let pos = self.pos()?;
        if pos < siblings.len() as i32 - 1 {
            self.selected = siblings[(pos + 1) as usize].clone();
        }
        Ok(())
    }

    pub fn move_up(&mut self) -> Result<()> {
        let siblings = self.siblings()?;
        let pos = self.pos()?;
        if pos > 0 {
            self.selected = siblings[(pos - 1) as usize].clone();
        }
        Ok(())
    }

    pub fn move_in(&mut self) -> Result<()> {
        if self.selected.is_dir() {
            self.paths.insert(self.cwd.clone(), self.selected.clone());
            self.cwd = self
                .cwd
                .clone()
                .join(self.selected.file_name().unwrap_or(OsStr::new("")));
            self.selected = self
                .paths
                .get(&self.cwd)
                .unwrap_or(&self.first_child()?)
                .clone();
        }
        std::env::set_current_dir(&self.cwd)?;
        Ok(())
    }

    pub fn move_out(&mut self) -> Result<()> {
        self.paths.insert(self.cwd.clone(), self.selected.clone());
        let child_cwd = self.cwd.clone();
        if self.cwd.parent().is_some() {
            self.cwd = self.cwd.parent().unwrap_or(&child_cwd).to_path_buf();
            self.selected = match self.paths.get(&self.cwd) {
                Some(v) => v.clone(),
                None => child_cwd,
            }
        }
        std::env::set_current_dir(&self.cwd)?;
        Ok(())
    }
}
