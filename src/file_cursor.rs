use std::{
    collections::HashMap,
    ffi::OsStr,
    path::{Path, PathBuf},
    time::UNIX_EPOCH,
};

use anyhow::Result;

use crate::cursor::{Cursor, Sort};

pub struct FileCursor {
    hide: bool,
    casing: bool,
    sort: Sort,
    paths: HashMap<PathBuf, PathBuf>,
    start_cwd: Option<PathBuf>,
    selected: PathBuf,
}

impl FileCursor {
    pub fn new() -> Self {
        Self {
            hide: true,
            casing: false,
            sort: Sort::Name,
            paths: HashMap::new(),
            start_cwd: None,
            selected: PathBuf::new(),
        }
    }
}

impl Cursor for FileCursor {
    fn init(&mut self, cwd: &Path) -> Result<()> {
        self.start_cwd = Some(cwd.to_path_buf().clone());
        self.paths = HashMap::new();
        self.selected = if let Some(p) = self.siblings(std::env::current_dir()?)?.first() {
            p.clone()
        } else {
            std::env::current_dir()?.join(PathBuf::from(".."))
        };
        Ok(())
    }

    fn move_down(&mut self, n: i32) -> Result<()> {
        let siblings = self.siblings(self.current_dir())?;
        let pos = self.pos()?;
        if pos < siblings.len() as i32 - n {
            self.selected = siblings[(pos + n) as usize].clone();
        } else {
            self.selected = siblings.last().unwrap_or(&self.selected).clone()
        }
        Ok(())
    }

    fn move_up(&mut self, n: i32) -> Result<()> {
        let siblings = self.siblings(self.current_dir())?;
        let pos = self.pos()?;
        if pos >= n {
            self.selected = siblings[(pos - n) as usize].clone();
        } else {
            self.selected = siblings.first().unwrap_or(&self.selected).clone()
        }
        Ok(())
    }

    fn move_in(&mut self) -> Result<()> {
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

    fn move_out(&mut self) -> Result<()> {
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

    fn move_bottom(&mut self) -> Result<()> {
        let siblings = self.siblings(self.current_dir())?;
        self.selected = siblings[siblings.len() - 1].clone();
        Ok(())
    }

    fn move_top(&mut self) -> Result<()> {
        let siblings = self.siblings(self.current_dir())?;
        self.selected = siblings[0].clone();
        Ok(())
    }

    fn toggle_hidden_files(&mut self) -> Result<()> {
        self.hide = !self.hide;
        Ok(())
    }

    fn toggle_case_sensitivity(&mut self) -> Result<()> {
        self.casing = !self.casing;
        Ok(())
    }

    fn sort_dir(&mut self) -> Result<()> {
        self.sort = Sort::Dir;
        Ok(())
    }

    fn sort_name(&mut self) -> Result<()> {
        self.sort = Sort::Name;
        Ok(())
    }

    fn sort_size(&mut self) -> Result<()> {
        self.sort = Sort::Size;
        Ok(())
    }

    fn sort_time(&mut self) -> Result<()> {
        self.sort = Sort::Time;
        Ok(())
    }

    fn search(&mut self, pattern: &str) -> Result<()> {
        let matches = self.matching_siblings(pattern)?;
        if !matches.is_empty() {
            if let Some(path) = matches.first() {
                self.selected = path.into()
            }
        }
        Ok(())
    }

    fn matching_siblings(&mut self, pattern: &str) -> Result<Vec<PathBuf>> {
        let siblings = self.siblings(self.current_dir())?;
        let mut matches = Vec::new();
        for sibling in siblings {
            if sibling
                .to_str()
                .unwrap_or("")
                .to_lowercase()
                .contains(&pattern.to_lowercase())
            {
                matches.push(sibling);
            }
        }
        Ok(matches)
    }

    fn selected(&self) -> PathBuf {
        self.selected.clone()
    }

    fn current_dir(&self) -> PathBuf {
        PathBuf::from(&self.selected().parent().unwrap_or("".as_ref()))
    }

    fn start_dir(&self) -> PathBuf {
        self.start_cwd.clone().unwrap_or_default()
    }

    fn parent(&self) -> PathBuf {
        self.current_dir()
            .parent()
            .unwrap_or(&PathBuf::from(""))
            .to_path_buf()
    }

    fn current_siblings(&mut self) -> Result<Option<Vec<PathBuf>>> {
        let siblings = self.siblings(self.current_dir())?;
        if !siblings.is_empty() {
            Ok(Some(siblings))
        } else {
            Ok(None)
        }
    }

    fn siblings(&mut self, path: PathBuf) -> Result<Vec<PathBuf>> {
        let mut siblings = Vec::new();
        for entry in std::fs::read_dir(path)? {
            let path = &entry?.path();
            if self.hide && self.hidden(path) {
                continue;
            } else {
                siblings.push(path.clone())
            }
        }
        match self.sort {
            Sort::Dir => self.sort_by_dir(&mut siblings),
            Sort::Name => self.sort_by_name(&mut siblings),
            Sort::Size => self.sort_by_size(&mut siblings),
            Sort::Time => self.sort_by_time(&mut siblings),
        }
        match self.casing {
            true => self.sort_by_casing(&mut siblings),
            false => (),
        }
        Ok(siblings)
    }

    fn hidden(&self, path: &Path) -> bool {
        path.file_name()
            .unwrap_or(OsStr::new(""))
            .to_str()
            .unwrap_or("")
            .starts_with('.')
    }

    fn sort_by_casing(&self, siblings: &mut [PathBuf]) {
        if self.casing {
            siblings.sort_by(|a, b| {
                a.to_str()
                    .unwrap_or("")
                    .to_lowercase()
                    .cmp(&b.to_str().unwrap_or("").to_lowercase())
            });
        }
    }

    fn sort_by_dir(&self, siblings: &mut [PathBuf]) {
        siblings.sort_by_key(|b| std::cmp::Reverse(b.is_dir()));
    }

    fn sort_by_name(&self, siblings: &mut [PathBuf]) {
        siblings.sort_by(|a, b| a.file_name().cmp(&b.file_name()));
    }

    fn sort_by_time(&self, siblings: &mut [PathBuf]) {
        siblings.sort_by(|a, b| {
            let a_mtime = a
                .metadata()
                .map_or(UNIX_EPOCH, |m| m.modified().unwrap_or(UNIX_EPOCH));
            let b_mtime = b
                .metadata()
                .map_or(UNIX_EPOCH, |m| m.modified().unwrap_or(UNIX_EPOCH));
            a_mtime.cmp(&b_mtime)
        });
    }

    fn sort_by_size(&self, siblings: &mut [PathBuf]) {
        siblings.sort_by(|a, b| {
            let a_len = a.metadata().map(|m| m.len()).unwrap_or(0);
            let b_len = b.metadata().map(|m| m.len()).unwrap_or(0);
            a_len.cmp(&b_len)
        });
    }

    fn pos(&mut self) -> Result<i32> {
        let pos = self
            .siblings(self.current_dir())?
            .iter()
            .position(|p| {
                // println!("{:?}", p);
                // println!("{:?}", &self.selected);
                p == &self.selected
            })
            .unwrap_or(0) as i32;
        Ok(pos)
    }
}

impl Default for FileCursor {
    fn default() -> Self {
        Self::new()
    }
}
