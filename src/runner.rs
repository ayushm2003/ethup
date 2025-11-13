use std::process::Stdio;
use tokio::{
    process::{Child, Command},
    signal,
};

use crate::config::{ClConfig, ElConfig};

pub fn spawn_el(cfg: &ElConfig) -> anyhow::Result<Child> {
    std::fs::create_dir_all(&cfg.data_dir)?;

    let child = Command::new(&cfg.bin)
        .arg("node")
        .arg("--chain")
        .arg(&cfg.chain)
        .arg("--datadir")
        .arg(&cfg.data_dir)
        .arg("--authrpc.addr")
        .arg(&cfg.authrpc_addr)
        .arg("--authrpc.port")
        .arg(cfg.authrpc_port.to_string())
        .arg("--authrpc.jwtsecret")
        .arg(&cfg.jwt_path)
        .arg("--http")
        .arg("--http.addr")
        .arg(&cfg.http_addr)
        .arg("--http.port")
        .arg(cfg.http_port.to_string())
        .arg("--http.api")
        .arg("all")
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()?;

    Ok(child)
}

pub fn spawn_cl(cfg: &ClConfig) -> anyhow::Result<Child> {
    std::fs::create_dir_all(&cfg.data_dir)?;

    let mut cmd = Command::new(&cfg.bin);
    cmd.arg("bn")
        .arg("--network")
        .arg(&cfg.chain)
        .arg("--datadir")
        .arg(&cfg.data_dir)
        .arg("--execution-endpoint")
        .arg(&cfg.execution_endpoint)
        .arg("--execution-jwt")
        .arg(&cfg.execution_jwt)
        .arg("--http");

    if let Some(ref url) = cfg.checkpoint_sync_url {
        cmd.arg("--checkpoint-sync-url").arg(url);
    }

    let child = cmd
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()?;

    Ok(child)
}

pub async fn start_nodes(el: &mut Child, cl: &mut Child) -> anyhow::Result<()> {
    tokio::select! {
        _ = signal::ctrl_c()  => {
            eprintln!("Ctrl+C recieved, shutting down clients...");

            if let Some(id) = el.id() {
                eprintln!("Killing EL pid {}", id);
                let _ = el.kill().await;
            }

            if let Some(id) = cl.id() {
                eprintln!("Killing CL pid {}", id);
                let _ = cl.kill().await;
            }
        },

        status = el.wait() => {
            let status = status?;
            eprintln!("EL exited with status {}", status);

            let _ = cl.kill().await;
            return Err(anyhow::anyhow!("EL exited unexpectedly"));
        },

        status = cl.wait() => {
            let status = status?;
            eprintln!("CL exited with status {}", status);

            let _ = el.kill().await;
            return Err(anyhow::anyhow!("CL exited unexpectedly"));
        },
    }

    Ok(())
}
