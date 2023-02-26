use std::{
    collections::HashMap,
    ffi::OsStr,
    path::{Path, PathBuf},
};

use crossterm::Result;

pub struct Cursor {
    hide: bool,
    paths: HashMap<PathBuf, PathBuf>,
    selected: PathBuf,
}

impl Cursor {
    pub fn new() -> Cursor {
        Cursor {
            hide: true,
            paths: HashMap::new(),
            selected: PathBuf::new(),
        }
    }

    pub fn init(&mut self) -> Result<()> {
        self.paths = HashMap::new();
        self.selected = if let Some(p) = self.siblings(std::env::current_dir()?)?.first() {
            p.clone()
        } else {
            std::env::current_dir()?.join(PathBuf::from(".."))
        };
        Ok(())
    }

    pub fn move_down(&mut self) -> Result<()> {
        let siblings = self.siblings(self.current_dir())?;
        let pos = self.pos()?;
        if pos < siblings.len() as i32 - 1 {
            self.selected = siblings[(pos + 1) as usize].clone();
        }
        Ok(())
    }

    pub fn move_up(&mut self) -> Result<()> {
        let siblings = self.siblings(self.current_dir())?;
        let pos = self.pos()?;
        if pos > 0 {
            self.selected = siblings[(pos - 1) as usize].clone();
        }
        Ok(())
    }

    pub fn move_in(&mut self) -> Result<()> {
        if self.selected().is_dir()
            && self.selected() != self.current_dir()
            && !self.selected().ends_with("..")
        {
            self.paths.insert(self.current_dir(), self.selected());
            self.selected = if let Some(p) = self.paths.get(&self.selected()) {
                p.clone()
            } else {
                match self.siblings(self.selected())?.first() {
                    Some(p) => p.clone(),
                    None => self.selected().join(PathBuf::from("..")),
                }
            };
            std::env::set_current_dir(self.current_dir())?;
        }
        Ok(())
    }

    pub fn move_out(&mut self) -> Result<()> {
        if self.current_dir().parent().is_some() {
            self.paths.insert(self.current_dir(), self.selected());
            self.selected = match self.paths.get(&self.parent()) {
                Some(p) => p.clone(),
                None => self.current_dir(),
            };
            std::env::set_current_dir(self.current_dir())?;
        }
        Ok(())
    }

    pub fn move_bottom(&mut self) -> Result<()> {
        let siblings = self.siblings(self.current_dir())?;
        self.selected = siblings[siblings.len() - 1].clone();
        Ok(())
    }

    pub fn move_top(&mut self) -> Result<()> {
        let siblings = self.siblings(self.current_dir())?;
        self.selected = siblings[0].clone();
        Ok(())
    }

    pub fn toggle_hidden_files(&mut self) -> Result<()> {
        self.hide = !self.hide;
        Ok(())
    }

    pub fn selected(&self) -> PathBuf {
        self.selected.clone()
    }

    pub fn current_dir(&self) -> PathBuf {
        PathBuf::from(&self.selected().parent().unwrap_or("".as_ref()))
    }

    pub fn parent(&self) -> PathBuf {
        self.current_dir()
            .parent()
            .unwrap_or(&PathBuf::from(""))
            .to_path_buf()
    }

    pub fn current_siblings(&self) -> Result<Vec<PathBuf>> {
        self.siblings(self.current_dir())
    }

    pub fn siblings(&self, path: PathBuf) -> Result<Vec<PathBuf>> {
        let mut siblings = Vec::new();
        for entry in std::fs::read_dir(path)? {
            let path = &entry?.path();
            if self.hide && self.hidden(path) {
                continue;
            } else {
                siblings.push(path.clone())
            }
        }
        siblings.sort();
        Ok(siblings)
    }

    fn hidden(&self, path: &Path) -> bool {
        path.file_name()
            .unwrap_or(OsStr::new(""))
            .to_str()
            .unwrap_or("")
            .starts_with('.')
    }

    pub fn pos(&self) -> Result<i32> {
        let pos = self
            .siblings(self.current_dir())?
            .iter()
            .position(|p| p == &self.selected)
            .unwrap_or(0) as i32;
        Ok(pos)
    }
}
