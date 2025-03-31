use tokio::process::Command;
use serde_json::{Result, Value};
use std::collections::HashSet;
use std::result::Result as StdResult;
use std::error::Error;

#[tokio::main]
async fn main() -> StdResult<(),  Box<dyn Error>> {
    let output = Command::new("swaymsg")
        .arg("-t")
        .arg("get_tree")
        .output()
        .await?;
    let json_str = String::from_utf8_lossy(&output.stdout).as_ref().to_string();
    let v: Value = serde_json::from_str(json_str.as_ref())?;
    let nodes = &v["nodes"];
    if let Value::Array(arr) = nodes {
        for entry in arr {
            if let Value::Array(arr2) = &entry["nodes"] {
                for entry2 in arr2 {
                    let maybe_number = entry2.get("num")
                        .and_then(|v| v.as_i64());
                    if let Value::Array(arr3) = &entry2["nodes"] {
                        for entry3 in arr3 {
                            //println!("\n\n\n\n{}",  entry3["name"]);
                            let maybe_app_name = entry3.get("name");
                            
                            let maybe_app_id = entry3.get("app_id");
                            match (maybe_number, maybe_app_id, maybe_app_name) {
                                (Some(num), Some(app_id), Some(app_name)) => {
                                    println!("{} | {} | {}", num, app_id, app_name);
                                }
                                _ => {}
                            }
                        }
                    }
                }
            } 
        }
    }


    Ok(())
}
