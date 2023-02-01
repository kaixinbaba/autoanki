mod cli;
mod config;
mod anki;

use anyhow::Result;

use cli::parse_args;


fn main() -> Result<()> {
    println!("hello world");
    let config = parse_args()?;
    println!("{:?}", config);
    Ok(())
}