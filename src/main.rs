use std::io;

mod cursor;
mod file_explorer;

fn main() -> anyhow::Result<()> {
    let mut stdout = io::stdout();
    file_explorer::run(&mut stdout)?;
    Ok(())
}
