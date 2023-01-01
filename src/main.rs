use std::env;
use std::io::Result;

fn main() -> Result<()> {
    print_current_dir()?;
    println!("");
    print_current_dir_content()?;
    Ok(())
}

fn print_current_dir() -> Result<()> {
    let current_dir = env::current_dir()?;
    println!("{}", current_dir.display());
    Ok(())
}

fn print_current_dir_content() -> Result<()> {
    for entry in std::fs::read_dir(".")? {
        let entry = entry?;
        let path = entry.path();
        let dir = path.file_name().unwrap().to_str().unwrap();
        if path.is_dir() {
            println!("   {}/", dir);
        } else {
            println!("   {}", dir);
        }
    }
    Ok(())
}
