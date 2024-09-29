use std::{
    collections::HashMap,
    ffi::OsStr,
    fs::File,
    io::Read,
    path::{Path, PathBuf},
    time::UNIX_EPOCH,
};

use anyhow::Result;
use tar::Archive;

use crate::cursor::{Cursor, Sort};

pub struct TarCursor {
    hide: bool,
    casing: bool,
    sort: Sort,
    archive: Option<Archive<File>>,
    start_cwd: Option<PathBuf>,
    tree: HashMap<PathBuf, Vec<PathBuf>>,
    paths: HashMap<PathBuf, PathBuf>,
    selected: PathBuf,
}

impl TarCursor {
    pub fn new() -> Self {
        Self {
            hide: true,
            casing: false,
            sort: Sort::Name,
            archive: None,
            start_cwd: None,
            tree: HashMap::new(),
            paths: HashMap::new(),
            selected: PathBuf::new(),
        }
    }

    pub fn read_file_content(&mut self, path: &Path) -> Result<Vec<u8>> {
        let relative_path = path.strip_prefix(&self.start_dir())?;
        let mut archive = Archive::new(File::open(&self.start_dir())?);
        let mut entry = archive
            .entries()?
            .filter_map(|e| e.ok())
            .find(|e| e.path().ok().map(|p| p == relative_path).unwrap_or(false))
            .ok_or_else(|| anyhow::anyhow!("File not found in archive"))?;

        let mut buffer = Vec::new();
        entry.read_to_end(&mut buffer)?;
        Ok(buffer)
    }
}

use std::collections::hash_map::Entry;

impl Cursor for TarCursor {
    fn init(&mut self, cwd: &Path) -> Result<()> {
        if self.start_cwd != Some(cwd.to_path_buf().clone()) {
            self.paths = HashMap::new();
            self.start_cwd = Some(cwd.to_path_buf().clone());
            if let Some(file_path) = cwd.to_str() {
                let file = File::open(file_path)?;
                self.archive = Some(Archive::new(file));
            } else {
                return Ok(());
            }

            // populate tree
            let mut tree: HashMap<PathBuf, Vec<PathBuf>> = HashMap::new();
            match &mut self.archive {
                Some(archive) => {
                    for entry in archive.entries()? {
                        let entry = entry?;
                        let path = entry.path()?.to_path_buf();

                        let parent = if path.parent() == Some(Path::new("")) {
                            cwd.to_str().unwrap_or("").into()
                        } else {
                            let cwd_str = cwd.to_str().unwrap_or("");
                            let parent_str = path.parent().and_then(|p| p.to_str()).unwrap_or("");
                            PathBuf::from(format!("{}/{}/", cwd_str, parent_str))
                        };

                        let file = cwd
                            .to_str()
                            .and_then(|cwd_str| {
                                path.to_str().map(|path_str| {
                                    if path_str.ends_with('/') {
                                        PathBuf::from(format!("{}/{}/", cwd_str, path_str))
                                    } else {
                                        PathBuf::from(format!("{}/{}", cwd_str, path_str))
                                    }
                                })
                            })
                            .unwrap_or_else(|| PathBuf::from(cwd));

                        if let Entry::Vacant(e) = tree.entry(parent.clone()) {
                            e.insert(vec![file]);
                        } else {
                            tree.entry(parent).or_insert(Vec::new()).push(file);
                        }
                    }
                }
                None => (),
            }

            self.tree = tree;

            self.selected = match self.start_cwd.as_ref() {
                Some(start_cwd) => match self.siblings(start_cwd.to_path_buf()) {
                    Ok(siblings) => siblings
                        .first()
                        .map(|s| s.to_path_buf())
                        .unwrap_or_else(|| PathBuf::from("")),
                    Err(_) => PathBuf::from(""),
                },
                None => PathBuf::from(""),
            };
        }

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
        if self
            .selected()
            .to_str()
            .map(|s| s.ends_with('/'))
            .unwrap_or(false)
            && self.selected() != self.current_dir()
            && !self.selected().ends_with("..")
        {
            self.paths.insert(self.current_dir(), self.selected());
            let selected = if let Some(p) = self.paths.get(&self.selected()) {
                p.clone()
            } else {
                match self.siblings(self.selected())?.first() {
                    Some(p) => p.clone(),
                    None => self.selected().join(PathBuf::from("..")),
                }
            };
            self.selected = selected
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
        let siblings = if let Some(s) = self.tree.get(&path) {
            s.clone()
        } else {
            Vec::new()
        };
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
            .position(|p| self.selected.file_name() == p.file_name())
            .unwrap_or(0) as i32;
        Ok(pos)
    }

    fn as_any_mut(&mut self) -> &mut dyn std::any::Any {
        self
    }
}

impl Default for TarCursor {
    fn default() -> Self {
        Self::new()
    }
}
