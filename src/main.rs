use std::io;

pub mod file_cursor;
pub mod tar_cursor;
pub mod engine;
pub mod explorer;
pub mod lines;
pub mod cursor;

fn main() -> anyhow::Result<()> {
    let mut stdout = io::stdout();
    explorer::run(&mut stdout)?;
    Ok(())
}
