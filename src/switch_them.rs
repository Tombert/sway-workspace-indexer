use crate::types;
use tokio::process::Command;
use std::{collections::HashMap, error::Error, future::Future};
use csv::ReaderBuilder;
use tokio::io::{self, AsyncBufReadExt, BufReader};

use std::result::Result as StdResult;

pub const DEBUG_URL : &str = "http://localhost:9222";


fn parse_pipe_delimited_line(line: &str) -> Vec<String> {
    let sanitized = line.split('|').map(str::trim).collect::<Vec<_>>().join("|");
    let mut rdr = ReaderBuilder::new()
        .delimiter(b'|')
        .has_headers(false)
        .from_reader(sanitized.as_bytes());

    rdr.records()
        .next()
        .unwrap()
        .unwrap()
        .iter()
        .map(|s| s.trim().to_string())
        .collect()
}

fn make_handler<F, Fut>(f: F) -> types::HandlerFn
where
    F: Fn(Vec<String>) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = StdResult<(), Box<dyn Error + Send + Sync>>> + Send + 'static,
{
    Box::new(move |args| Box::pin(f(args)))
}


async fn tmux_handler(my_line: Vec<String>) -> StdResult<(), Box<dyn Error + Send + Sync>> {
    let id = &my_line[3];
    let tty = &my_line[1];
    let workspace = &my_line[0];

    let full_cmd = format!(
        "tmux select-window -t {} \\; select-pane -t {}",
        workspace, id
    );
    let resp1 = Command::new("sh").arg("-c").arg(&full_cmd).output();

    let resp2 = Command::new("swaymsg")
        .arg(format!("[app_id=\"{}\"] focus", tty))
        .output();

    let (_resp1, _resp2) = tokio::join!(resp1, resp2);

    Ok(())
}


async fn tab_handler(my_line: Vec<String>) -> StdResult<(), Box<dyn Error + Send + Sync>> {
    let browser = &my_line[1];
    let id = &my_line[3];
    let client = reqwest::Client::new();

    let s = format!("{}/json/activate/{}", DEBUG_URL, id);
    let resp1 = client.post(s).send();

    let resp2 = Command::new("swaymsg")
        .arg(format!("[app_id=\"{}\"] focus", browser))
        .output();

    let (_resp1, _resp2) = tokio::join!(resp1, resp2);

    Ok(())
}

async fn default_handler(my_line: Vec<String>) -> StdResult<(), Box<dyn Error + Send + Sync>> {
    let app = &my_line[1];
    let title = &my_line[2];
    let real_title = (!title.is_empty())
        .then(|| format!(" title=\"{}\"", title))
        .unwrap_or_default();

    let arg_str = format!("[app_id=\"{}\"{}] focus", app, real_title);
    let _ = Command::new("swaymsg")
        .arg(arg_str)
        //.arg("focus'")
        .output()
        .await?;
    Ok(())
}

pub async fn switch_apps(_args : types::Args ) -> StdResult<(), Box<dyn Error>> {
    let map: HashMap<String, types::HandlerFn> = vec![
        ("tmux".to_string(), make_handler(tmux_handler)),
        ("tab".to_string(), make_handler(tab_handler)),
    ]
    .into_iter()
    .collect();
    let stdin = BufReader::new(io::stdin());
    let mut lines = stdin.lines();

    while let Ok(Some(line)) = lines.next_line().await {
        let my_line = parse_pipe_delimited_line(line.as_ref());
        let default = make_handler(default_handler);
        let handler = map.get(&my_line[4]).unwrap_or(&default);
        let _ = handler(my_line).await;
        break;
    }
    Ok(())
}
