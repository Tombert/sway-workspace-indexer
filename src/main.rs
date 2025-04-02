mod new_ws;
mod switch_them;
mod get_apps;
use clap::Parser;
use std::result::Result as StdResult;
use std::error::Error;


#[derive(Parser)]
#[command(name = "sway-app-workspace-index")]
#[command(author = "thomas@gebert.app")]
#[command(version = "1.0")]
#[command(about = "nada")]
struct Args {
    #[arg(short, long)]
    command: String,
}


#[tokio::main]
async fn main() -> StdResult<(), Box<dyn Error>> {
    let args = Args::parse();

    match args.command.as_str() {
        "get-apps" => get_apps::get_all_apps().await?,
        "switch-apps" => switch_them::switch_apps().await?,
        "new-ws" => new_ws::do_new_workspace().await?,
        _ => (),
    }
    Ok(())
}
