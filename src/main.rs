mod get_apps;
mod new_ws;
mod switch_them;
use clap::Parser;
use std::error::Error;
use std::result::Result as StdResult;
mod types;

#[tokio::main]
async fn main() -> StdResult<(), Box<dyn Error>> {
    let args = types::Args::parse();

    match args.command.as_str() {
        "get-apps" => get_apps::get_all_apps(args).await?,
        "switch-apps" => switch_them::switch_apps(args).await?,
        "new-ws" => new_ws::do_new_workspace(args).await?,
        _ => (),
    }
    Ok(())
}
