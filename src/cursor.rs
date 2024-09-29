use std::path::{Path, PathBuf};

pub enum Sort {
    Dir,
    Name,
    Size,
    Time,
}

pub(crate) trait Cursor {
    fn init(&mut self, cwd: &Path) -> anyhow::Result<()>;
    fn move_down(&mut self, n: i32) -> anyhow::Result<()>;
    fn move_up(&mut self, n: i32) -> anyhow::Result<()>;
    fn move_in(&mut self) -> anyhow::Result<()>;
    fn move_out(&mut self) -> anyhow::Result<()>;
    fn move_bottom(&mut self) -> anyhow::Result<()>;
    fn move_top(&mut self) -> anyhow::Result<()>;
    fn toggle_hidden_files(&mut self) -> anyhow::Result<()>;
    fn toggle_case_sensitivity(&mut self) -> anyhow::Result<()>;
    fn sort_dir(&mut self) -> anyhow::Result<()>;
    fn sort_name(&mut self) -> anyhow::Result<()>;
    fn sort_size(&mut self) -> anyhow::Result<()>;
    fn sort_time(&mut self) -> anyhow::Result<()>;
    fn search(&mut self, pattern: &str) -> anyhow::Result<()>;
    fn matching_siblings(&mut self, pattern: &str) -> anyhow::Result<Vec<PathBuf>>;
    fn selected(&self) -> PathBuf;
    fn current_dir(&self) -> PathBuf;
    fn start_dir(&self) -> PathBuf;
    fn parent(&self) -> PathBuf;
    fn current_siblings(&mut self) -> anyhow::Result<Option<Vec<PathBuf>>>;
    fn siblings(&mut self, path: PathBuf) -> anyhow::Result<Vec<PathBuf>>;
    fn hidden(&self, path: &Path) -> bool;
    fn sort_by_casing(&self, siblings: &mut [PathBuf]);
    fn sort_by_dir(&self, siblings: &mut [PathBuf]);
    fn sort_by_name(&self, siblings: &mut [PathBuf]);
    fn sort_by_time(&self, siblings: &mut [PathBuf]);
    fn sort_by_size(&self, siblings: &mut [PathBuf]);
    fn pos(&mut self) -> anyhow::Result<i32>;
    fn as_any(&self) -> &dyn std::any::Any;
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any;
}
