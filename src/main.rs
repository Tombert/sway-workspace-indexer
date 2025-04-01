use tokio::process::Command;
use serde_json::{Result, Value};
use std::collections::HashSet;
use std::result::Result as StdResult;
use std::error::Error;
use std::future::Future;

#[tokio::main]
async fn main() -> StdResult<(),  Box<dyn Error>> {

    let tabs_future = reqwest::get("http://localhost:9222/json");


    let output_future = Command::new("swaymsg")
        .arg("-t")
        .arg("get_tree")
        .output();
    let (tabs_resp, output) = tokio::join!(tabs_future, output_future);

    let tabs = match tabs_resp {
        Ok(resp) => resp.json::<Value>().await.unwrap_or(Value::Array(vec![])),
        Err(_) => Value::Array(vec![]),
    };

    let b : Vec<(i64, &str, String, String, &str)> = if let Value::Array(arr) = tabs {
        arr.iter().filter_map(|i| 
            if  i["type"] == "page" {
                let id = i.get("id")?.to_string();
                let title = i.get("title")?.to_string();
               Some((1000, "brave-browser", title ,id , "tab"))
            } else {
                None
            }).collect()
    } else {
        Vec::new()
    };
    let output = output?;
    let json_str = String::from_utf8_lossy(&output.stdout).as_ref().to_string();
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
                        Some((num, app_id, app_name, "".to_string(), "app"))
                    }))
                }))
        .flatten().collect()
    } else {
        Vec::new()
    };

    let full_list = [b, apps].concat(); 
    full_list.iter().for_each(|(num, app_id, app_name, id, ttype)|{
        println!("{} | {} | {} | {} | {}", num, app_id, app_name, id, ttype)
    });

    Ok(())
}
