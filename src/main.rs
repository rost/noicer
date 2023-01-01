use std::env;

fn main() -> std::io::Result<()> {
    print_current_dir()?;
    Ok(())
}

fn print_current_dir() -> std::io::Result<()> {
    let current_dir = env::current_dir()?;
    println!("{}", current_dir.display());
    Ok(())
}
