
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

