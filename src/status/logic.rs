use reqwest::Client;
use serde::de::DeserializeOwned;
use serde_json::{Value, json};

use super::types::*;
use crate::config::{ClConfig, ElConfig};

pub async fn status(el: &ElConfig, cl: &ClConfig) -> anyhow::Result<()> {
    let el_status = el_status(el).await?;
    let cl_status = cl_status(cl).await?;

    println!("Execution Client Running:");
    println!("  Version: {}", el_status.version);
    println!("  Chain ID: {}", el_status.chain_id);
    println!("  Executed Blocks: {}", el_status.head_block);
    println!("  Sync: {}", el_status.sync);

    println!();
    println!("Consensus Client Running:");

    println!("  Version: {}", cl_status.version);
    println!("  Head slot: {}", cl_status.head_slot);

    if let Some(finalized_epoch) = cl_status.finalized_epoch {
        println!("  Finalized epoch: {}", finalized_epoch);
    }

    println!(
        "  Sync: {}",
        if cl_status.is_syncing {
            "syncing"
        } else {
            "not syncing"
        }
    );

    println!("  Health: {}", cl_status.health);

    Ok(())
}

pub async fn el_status(el: &ElConfig) -> anyhow::Result<ExecutionStatus> {
    let version: String = el_rpc(el, "web3_clientVersion", json!([]))
        .await?
        .as_str()
        .unwrap_or("?")
        .to_string();

    let chain_id_hex: String = el_rpc(el, "eth_chainId", json!([]))
        .await?
        .as_str()
        .unwrap_or("?")
        .to_string();

    let head_hex: String = el_rpc(el, "eth_blockNumber", json!([]))
        .await?
        .as_str()
        .unwrap_or("?")
        .to_string();

    let syncing: ElSyncing = {
        let raw = el_rpc(el, "eth_syncing", json!([])).await?;
        serde_json::from_value(raw)?
    };

    let chain_id = parse_hex_u64(&chain_id_hex)?;
    let head_block = parse_hex_u64(&head_hex)?;

    let sync_state = match syncing {
        ElSyncing::NotSyncing(_) => ElSyncState::FullySynced,
        ElSyncing::Syncing {
            starting_block,
            current_block,
            highest_block,
        } => {
            let s = parse_hex_u64(&starting_block)?;
            let c = parse_hex_u64(&current_block)?;
            let h = parse_hex_u64(&highest_block)?;

            let percent = if h == 0 {
                0.0
            } else {
                (c as f64 / h as f64) * 100.0
            };

            ElSyncState::Syncing {
                starting_block: s,
                current_block: c,
                highest_block: h,
                percent,
            }
        }
    };

    Ok(ExecutionStatus {
        version,
        chain_id,
        head_block,
        sync: sync_state,
    })
}

async fn cl_status(cl: &ClConfig) -> anyhow::Result<ConsensusStatus> {
    let ver: ClApi<ClVersion> = cl_get(cl, "eth/v1/node/version").await?;
    let sync: ClApi<ClSync> = cl_get(cl, "eth/v1/node/syncing").await?;
    let health = cl_health(&cl.http_url()).await?;

    let head_slot = sync.data.head_slot.parse::<u64>()?;
    let finalized_epoch = match sync.data.finalized_epoch {
        Some(s) => Some(s.parse::<u64>()?),
        None => None,
    };

    Ok(ConsensusStatus {
        version: ver.data.version,
        head_slot,
        finalized_epoch,
        is_syncing: sync.data.is_syncing,
        health,
    })
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

async fn cl_get<T: DeserializeOwned>(cl: &ClConfig, path: &str) -> anyhow::Result<T> {
    let url = format!("{}/{}", cl.http_url(), path);
    let resp = reqwest::get(&url).await?;
    let status = resp.status();

    if !status.is_success() {
        anyhow::bail!("CL endpoint {} returned HTTP {}", &url, status);
    }

    let val = resp.json::<T>().await?;
    Ok(val)
}

async fn cl_health(cl_http_url: &str) -> anyhow::Result<ClHealth> {
    let url = format!("{}/eth/v1/node/health", cl_http_url);
    let resp = reqwest::get(url).await?;
    let code = resp.status().as_u16();

    let health = match code {
        200 => ClHealth::Healthy,
        206 => ClHealth::Syncing,
        503 => ClHealth::Unhealthy,
        other => ClHealth::Unknown(other),
    };

    Ok(health)
}

pub fn parse_hex_u64(s: &str) -> anyhow::Result<u64> {
    let s = s.trim_start_matches("0x");
    Ok(u64::from_str_radix(s, 16)?)
}
