use reqwest::Client;
use serde_json::{Value, json};

use crate::config::{ClConfig, ElConfig};

pub async fn status(el: &ElConfig, cl: &ClConfig) -> anyhow::Result<()> {
    // TODO: Use structs.
    let el_version = el_rpc(el, "web3_clientVersion", json!([])).await?;
    let el_syncing = el_rpc(el, "eth_syncing", json!([])).await?;
    let head_hex = el_rpc(el, "eth_blockNumber", json!([])).await?;
    let chain_id = el_rpc(el, "eth_chainId", json!([])).await?;

    let head = parse_hex_u64(&head_hex)?;

    let cl_version = cl_get(cl, "eth/v1/node/version").await?;
    let cl_syncing = cl_get(cl, "eth/v1/node/syncing").await?;

    println!("Execution Client Running:");
    println!("  Version: {}", el_version.as_str().unwrap_or("?"));
    println!("  Chain ID: {}", chain_id.as_str().unwrap_or("?"));
    println!("  Executed Blocks: {}", head);

    if el_syncing.is_object() {
        let cur = parse_hex_u64(&el_syncing["currentBlock"])?;
        let high = parse_hex_u64(&el_syncing["highestBlock"])?;

        if high == 0 {
            println!("  Sync: execution not started yet…");
        } else {
            let pct = (cur as f64 / high as f64) * 100.0;
            println!("  Sync: {cur}/{high} ({pct:.2}%)");
        }
    } else {
        println!("  Sync: Fully synced");
    }

    println!();
    println!("Consensus Client Running:");
    println!(
        "  Version: {}",
        cl_version["data"]["version"].as_str().unwrap_or("?")
    );
    println!(
        "  Head slot: {}",
        cl_syncing["data"]["head_slot"].as_str().unwrap_or("?")
    );
    println!(
        "  Finalized epoch: {}",
        cl_syncing["data"]["finalized_epoch"]
    );
    println!("  Syncing: {}", cl_syncing["data"]["is_syncing"]);

    print!("  Health: ");
    match cl_health(&cl.http_url()).await? {
        200 => println!("CL healthy"),
        206 => println!("CL syncing"),
        503 => println!("CL unhealthy"),
        code => println!("CL unknown status {}", code),
    }

    Ok(())
}

async fn el_rpc(el: &ElConfig, method: &str, params: Value) -> anyhow::Result<Value> {
    let client = Client::new();

    let payload = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "method": method,
        "params": params,
    });

    let resp = client
        .post(el.rpc_url())
        .json(&payload)
        .send()
        .await?
        .json::<Value>()
        .await?;

    Ok(resp["result"].clone())
}

async fn cl_get(cl: &ClConfig, path: &str) -> anyhow::Result<Value> {
    let url = format!("{}/{}", cl.http_url(), path);
    let resp = reqwest::get(&url).await?;
    let status = resp.status();

    if !status.is_success() {
        anyhow::bail!("CL endpoint {} returned HTTP {}", &url, status);
    }

    let val = resp.json::<Value>().await?;
    Ok(val)
}

async fn cl_health(cl_http_url: &str) -> anyhow::Result<u16> {
    let url = format!("{}/{}", cl_http_url, "eth/v1/node/health");
    let resp = reqwest::get(url).await?;
    Ok(resp.status().as_u16())
}

fn parse_hex_u64(val: &Value) -> anyhow::Result<u64> {
    let s = val.as_str().unwrap_or("0").trim_start_matches("0x");

    Ok(u64::from_str_radix(s, 16)?)
}
