use crate::switch_them;
use tokio::process::Command;
use serde_json::Value;
use std::result::Result as StdResult;
use std::error::Error;

fn get_tmux(tmux_str : String) -> Vec<(String, String, String, String, String)> {
    tmux_str
        .lines()
        .filter_map(|line| {
            let sections: Vec<&str> = line.split("|").collect();
            Some((
                sections[0].to_string(),
                "footclient".to_string(),
                format!("{} {}", sections[3], sections[4]),
                sections[2].to_string(),
                "tmux".to_string(),
            ))
        })
        .collect()
}


fn get_tabs(tabs : Value) -> Vec<(String, String, String, String, String)>{
   if let Value::Array(arr) = tabs {
        arr.iter()
            .filter_map(|i| {
                if i["type"] == "page" {
                    let id = i.get("id")?.to_string();
                    let title = i.get("title")?.to_string();
                    Some((1000.to_string(), "brave-browser".to_string(), title, id, "tab".to_string()))
                } else {
                    None
                }
            })
        .collect()
    } else {
        Vec::new()
    }
}

fn get_apps(v: Value) -> Vec<(String, String, String, String, String)> {
    let nodes = &v["nodes"];
    if let Value::Array(arr) = nodes {
        arr.iter()
            .filter_map(|entry| entry.get("nodes")?.as_array())
            .flat_map(|arr2| {
                arr2.iter().filter_map(|entry2| {
                    let num = entry2.get("num")?.as_i64()?;
                    let arr3 = entry2.get("nodes")?.as_array()?;
                    Some(arr3.iter().filter_map(move |entry3| {
                        let app_id = entry3.get("app_id")?;
                        let app_name = entry3.get("name")?;
                        Some((num.to_string(), app_id.to_string(), app_name.to_string(), "".to_string(), "app".to_string()))
                    }))
                })
            })
            .flatten()
            .collect()
    } else {
        Vec::new()
    }
}


pub async fn get_all_apps() -> StdResult<(), Box<dyn Error>> {
    let url = format!("{}/json", switch_them::DEBUG_URL);
    let tabs_future = reqwest::get(&url);

    let tmux_future = Command::new("tmux")
        .arg("list-panes")
        .arg("-a")
        .arg("-F")
        .arg("#{session_name}:#{window_index}|#{pane_index}|#{pane_id}|#{pane_current_path}|#{pane_current_command}|#{pane_active}|#{pane_pid}")
        .output();

    let apps_future = Command::new("swaymsg").arg("-t").arg("get_tree").output();

    let (tabs_resp, apps_resp, tmux_resp) = tokio::join!(tabs_future, apps_future, tmux_future);
    let tmux = tmux_resp?;
    let tmux_str = String::from_utf8_lossy(&tmux.stdout).as_ref().to_string();

    let tmux_arr = get_tmux(tmux_str);

    let tabs = match tabs_resp {
        Ok(resp) => resp.json::<Value>().await.unwrap_or(Value::Array(vec![])),
        Err(e) => {println!("Error: {}", e);  Value::Array(vec![])},
    };
    println!("Fuck {}: " ,tabs);



    let tabs = get_tabs(tabs);

    let apps = apps_resp?;
    let json_str = String::from_utf8_lossy(&apps.stdout).as_ref().to_string();
    let v: Value = serde_json::from_str(json_str.as_ref())?;
    let apps = get_apps(v);

    [tabs, tmux_arr, apps]
        .concat()
            .iter()
            .for_each(|(num, app_id, app_name, id, ttype)| {
                println!("{} | {} | {} | {} | {}", num, app_id, app_name, id, ttype)
            });

    Ok(())
}
