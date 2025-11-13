use std::path::Path;
use std::process::Stdio;
use tokio::{
    process::{Child, Command},
    signal,
};

use crate::layout::{bin_dir, data_dir};

pub fn spawn_reth(jwt_path: &Path) -> anyhow::Result<Child> {
    let reth_bin = bin_dir().join("reth");
    let reth_data = data_dir().join("reth");
    std::fs::create_dir_all(&reth_data)?;

    let child = Command::new(reth_bin)
        .arg("node")
        .arg("--chain")
        .arg("hoodi")
        .arg("--datadir")
        .arg(reth_data)
        .arg("--authrpc.addr")
        .arg("127.0.0.1")
        .arg("--authrpc.port")
        .arg("8551")
        .arg("--authrpc.jwtsecret")
        .arg(jwt_path)
        .arg("--http")
        .arg("--http.addr")
        .arg("127.0.0.1")
        .arg("--http.port")
        .arg("8545")
        .arg("--http.api")
        .arg("all")
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()?;

    Ok(child)
}

pub fn spawn_lighthouse(jwt_path: &Path) -> anyhow::Result<Child> {
    let lh_bin = bin_dir().join("lighthouse");
    let lh_data = data_dir().join("lighthouse");

    std::fs::create_dir_all(&lh_data)?;

    let child = Command::new(lh_bin)
        .arg("bn")
        .arg("--network")
        .arg("hoodi")
        .arg("--datadir")
        .arg(lh_data)
        .arg("--execution-endpoint")
        .arg("http://127.0.0.1:8551")
        .arg("--execution-jwt")
        .arg(jwt_path)
        .arg("--checkpoint-sync-url")
        .arg("https://checkpoint-sync.hoodi.ethpandaops.i")
        .arg("--http")
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
