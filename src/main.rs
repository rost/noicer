use std::io;

pub mod cursor;
pub mod engine;
pub mod file_explorer;
pub mod lines;

fn main() -> anyhow::Result<()> {
    let mut stdout = io::stdout();
    file_explorer::run(&mut stdout)?;
    Ok(())
}
