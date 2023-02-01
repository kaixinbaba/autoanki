use clap::Parser;

use crate::{config::Config};
use anyhow::Result;

#[derive(Parser, Debug)]
#[clap(
    author,
    version,
    about,
    long_about = "A toolkit for handle Anki quickly"
)]
pub struct Args {
    #[clap(short, long, help = "配置文件路径")]
    pub path: Option<String>,
}

pub fn parse_args() -> Result<Config> {
    let args = Args::parse();
    Ok(Config::from(args))
}
