use std::io;

pub mod cursor;
pub mod engine;
pub mod explorer;
pub mod file_cursor;
pub mod lines;
pub mod tar_cursor;

fn main() -> anyhow::Result<()> {
    let mut stdout = io::stdout();
    explorer::run(&mut stdout)?;
    Ok(())
}
