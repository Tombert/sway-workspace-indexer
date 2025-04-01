use tokio::process::Command;

use tokio::io::{self, AsyncBufReadExt, BufReader};
use serde_json::{Result, Value};
use std::{collections::HashMap, error::Error, future::Future, pin::Pin};
use std::result::Result as StdResult;
use csv::ReaderBuilder;
use clap::Parser;


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

async fn tab_handler(my_line: Vec<String>) -> StdResult<(), Box<dyn Error + Send + Sync>> {
    let id = &my_line[3];
    let client = reqwest::Client::new();

    let s = format!("http://localhost:9222/json/activate/{}", id);
    let resp1 = client.post(s).send();

    let resp2 = Command::new("swaymsg")
        .arg("[app_id=\"brave-browser\"] focus")
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

type HandlerFn = Box<
    dyn Fn(
            Vec<String>,
        )
            -> Pin<Box<dyn Future<Output = StdResult<(), Box<dyn Error + Send + Sync>>> + Send>>
        + Send
        + Sync,
>;

fn make_handler<F, Fut>(f: F) -> HandlerFn
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






/// My awesome CLI tool
#[derive(Parser)]
#[command(name = "sway-app-workspace-index")]
#[command(author = "thomas@gebert.app")]
#[command(version = "1.0")]
#[command(about = "nada")]
struct Args {
    /// Some input file
    #[arg(short, long)]
    command: String,

    /// Enable verbose mode
    #[arg(short, long)]
    verbose: bool,
}


async fn get_apps() -> StdResult<(),  Box<dyn Error>> {
    let tabs_future = reqwest::get("http://localhost:9222/json");

    let tmux_future = Command::new("tmux")
        .arg("list-panes")
        .arg("-a")
        .arg("-F")
        .arg("#{session_name}:#{window_index}|#{pane_index}|#{pane_id}|#{pane_current_path}|#{pane_current_command}|#{pane_active}|#{pane_pid}")
        .output();

    let apps_future = Command::new("swaymsg")
        .arg("-t")
        .arg("get_tree")
        .output();

    let (tabs_resp, apps_resp, tmux_resp) = tokio::join!(tabs_future, apps_future, tmux_future);
    let tmux = tmux_resp?;
    let tmux_str = String::from_utf8_lossy(&tmux.stdout).as_ref().to_string();

    let tmux_arr : Vec<(String, &str, String, String, &str)> = tmux_str.lines().filter_map(|line| {
        let sections : Vec<&str>= line.split("|").collect(); 
        Some((sections[0].to_string(), "footclient", format!("{} {}", sections[3], sections[4]), sections[2].to_string(), "tmux"))

    }).collect();

    let tabs = match tabs_resp {
        Ok(resp) => resp.json::<Value>().await.unwrap_or(Value::Array(vec![])),
        Err(_) => Value::Array(vec![]),
    };

    let b : Vec<(String, &str, String, String, &str)> = if let Value::Array(arr) = tabs {
        arr.iter().filter_map(|i| 
            if  i["type"] == "page" {
                let id = i.get("id")?.to_string();
                let title = i.get("title")?.to_string();
                Some((1000.to_string(), "brave-browser", title ,id , "tab"))
            } else {
                None
            }).collect()
    } else {
        Vec::new()
    };
    let apps = apps_resp?;
    let json_str = String::from_utf8_lossy(&apps.stdout).as_ref().to_string();
    let v: Value = serde_json::from_str(json_str.as_ref())?;
    let nodes = &v["nodes"];
    let apps = if let Value::Array(arr) = nodes {
        arr.iter()
            .filter_map(|entry| entry.get("nodes")?.as_array())
            .flat_map(|arr2| 
                arr2.iter().filter_map(|entry2| {
                    let num = entry2.get("num")?.as_i64()?;
                    let arr3 = entry2.get("nodes")?.as_array()?;
                    Some(arr3.iter().filter_map(move |entry3| {
                        let app_id = entry3.get("app_id")?.as_str()?;
                        let app_name = entry3.get("name")?.to_string();
                        Some((num.to_string(), app_id, app_name, "".to_string(), "app"))
                    }))
                }))
        .flatten().collect()
    } else {
        Vec::new()
    };

    let full_list = [b, tmux_arr, apps].concat(); 
    full_list.iter().for_each(|(num, app_id, app_name, id, ttype)|{
        println!("{} | {} | {} | {} | {}", num, app_id, app_name, id, ttype)
    });

    Ok(())
}

async fn switch_apps() -> StdResult<(),  Box<dyn Error>> {
    let map: HashMap<String, HandlerFn> = vec![
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


#[tokio::main]
async fn main() -> StdResult<(),  Box<dyn Error>> {
    let args = Args::parse();

    if args.command == "get-apps" {
        get_apps().await
    } else if args.command == "switch-apps" {
        switch_apps().await
    } else {

        Ok(())
    }
}
