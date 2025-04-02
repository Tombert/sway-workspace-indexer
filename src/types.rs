use std::result::Result as StdResult;
use std::{error::Error, future::Future, pin::Pin};

use clap::Parser;

#[derive(Parser)]
#[command(name = "sway-app-workspace-index")]
#[command(author = "thomas@gebert.app")]
#[command(version = "1.0")]
#[command(about = "nada")]
pub struct Args {
    #[arg(short, long)]
    pub command: String,

    #[arg(short, long)]
    pub browser: Option<String>,

    #[arg(short, long)]
    pub terminal: Option<String>,
}

pub type HandlerFn = Box<
    dyn Fn(
            Vec<String>,
        )
            -> Pin<Box<dyn Future<Output = StdResult<(), Box<dyn Error + Send + Sync>>> + Send>>
        + Send
        + Sync,
>;
