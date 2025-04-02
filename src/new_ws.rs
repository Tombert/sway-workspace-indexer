use serde_json::{Result, Value};
use std::result::Result as StdResult;
use std::{collections::HashSet, error::Error};
use tokio::process::Command;

use crate::types;

async fn create_workspace(work_num: i64) -> StdResult<(), Box<dyn Error>> {
    Command::new("swaymsg")
        .arg("workspace")
        .arg(work_num.to_string())
        .output()
        .await?;
    Ok(())
}

async fn get_workspace_json() -> StdResult<String, Box<dyn Error>> {
    let output = Command::new("swaymsg")
        .arg("-t")
        .arg("get_workspaces")
        .output()
        .await?;
    return Ok(String::from_utf8_lossy(&output.stdout).as_ref().to_string());
}

fn workspace_get_value(x: String) -> Result<Value> {
    let v: Value = serde_json::from_str(x.as_ref())?;
    return Ok(v);
}

pub async fn do_new_workspace(_args: types::Args) -> StdResult<(), Box<dyn Error>> {
    let output = get_workspace_json().await?;
    let v: Value = workspace_get_value(output)?;

    if let Value::Array(arr) = v {
        let b: HashSet<i64> = arr
            .iter()
            .filter_map(|workspace| workspace.get("num").and_then(Value::as_i64))
            .collect();
        for i in 1..20 {
            if !b.contains(&i) {
                let _ = create_workspace(i).await;
                break;
            }
        }
    }
    Ok(())
}
