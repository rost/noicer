use std::path::PathBuf;

use crate::cursor::Cursor;

pub struct Lines {}

impl Default for Lines {
    fn default() -> Self {
        Self::new()
    }
}

impl Lines {
    pub fn new() -> Self {
        Self {}
    }

    pub(crate) fn format(&self, cursor: &mut dyn Cursor) -> anyhow::Result<Vec<String>> {
        let rows = match cursor.current_siblings()? {
            Some(content) => content,
            None => vec![PathBuf::from("   ../")],
        };

        let mut lines = Vec::new();
        lines.push(format!("{}", cursor.current_dir().display()));
        lines.push(String::from(""));

        for path in rows {
            let s = path
                .file_name()
                .unwrap_or_default()
                .to_str()
                .unwrap_or_default();
            let s = match path.is_dir() || path.to_str().unwrap_or("").ends_with('/') {
                true => format!("   {s}/"),
                false => format!("   {s}"),
            };
            lines.push(s);
        }

        let index = (cursor.pos()? + 2) as usize;
        lines[index] = format!(" > {}", lines[index].trim_start());

        Ok(lines)
    }
}
