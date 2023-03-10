extern crate serde;
extern crate serde_json;

#[macro_use]
extern crate serde_derive;
extern crate core;


mod cli;
mod config;
mod http;

use std::fmt::format;
use std::time::Duration;
use anyhow::Result;
use futures::future::join_all;
use tokio::task;
use clipboard::{ClipboardProvider, ClipboardContext};
use colored::Colorize;
use tokio::time::sleep;

use cli::parse_args;


#[tokio::main]
async fn main() -> Result<()> {
    let (config, mut args) = parse_args()?;
    let words = if args.words.is_empty() {
        vec![get_clipboard_text().await]
    } else {
        args.words
    };
    let mut handlers = Vec::with_capacity(words.len());

    for word in words {
        handlers.push(task::spawn(async move {
            if let Err(e) = http::save_word(word.clone()).await {
                println!("[{}] {} detail: {}", "X".red(), word.red(), e.to_string());
            } else {
                println!("[{}] {}", "√".green(), word.green());
            }
        }));
    }

    join_all(handlers).await;

    Ok(())
}


async fn get_clipboard_text() -> String {
    task::spawn_blocking(|| {
        let mut ctx: ClipboardContext = ClipboardProvider::new().unwrap();
        ctx.get_contents().unwrap()
    }).await.unwrap()
}




