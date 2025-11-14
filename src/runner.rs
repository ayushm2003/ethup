use std::process::Stdio;

use tokio::{
    io::{AsyncBufReadExt, BufReader},
    process::{Child, ChildStdout, Command},
    signal,
};

use crate::config::{ClConfig, ElConfig};

pub fn spawn_el(cfg: &ElConfig, quiet: bool) -> anyhow::Result<Child> {
    std::fs::create_dir_all(&cfg.data_dir)?;

    let mut cmd = Command::new(&cfg.bin);
    cmd.arg("node")
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
        .spawn()?;

    if quiet {
        cmd.stdout(Stdio::null()).stderr(Stdio::null());
    } else {
        cmd.stdout(Stdio::piped()).stderr(Stdio::piped());
    }

    // let child = if *quiet {
    //     std::fs::create_dir_all(log_dir())?;
    //     let log = File::create(log_dir().join("el.log"))?;
    //     cmd.stdout(Stdio::from(log.try_clone()?))
    //         .stderr(Stdio::from(log))
    //         .spawn()?
    // } else {
    //     cmd.stdout(Stdio::inherit())
    //         .stderr(Stdio::inherit())
    //         .spawn()?
    // };

    Ok(cmd.spawn()?)
}

pub fn spawn_cl(cfg: &ClConfig, quiet: bool) -> anyhow::Result<Child> {
    std::fs::create_dir_all(&cfg.data_dir)?;

    let mut cmd = Command::new(&cfg.bin);
    cmd.arg("bn")
        .arg("--network")
        .arg(&cfg.chain)
        .arg("--listen-address")
        .arg("0.0.0.0")
        .arg("--datadir")
        .arg(&cfg.data_dir)
        .arg("--execution-endpoint")
        .arg(&cfg.execution_endpoint)
        .arg("--execution-jwt")
        .arg(&cfg.execution_jwt)
        .arg("--http")
        .arg("--http-address")
        .arg(&cfg.http_addr)
        .arg("--http-port")
        .arg(cfg.http_port.to_string());

    if let Some(ref url) = cfg.checkpoint_sync_url {
        cmd.arg("--checkpoint-sync-url").arg(url);
    }

    if quiet {
        cmd.stdout(Stdio::null()).stderr(Stdio::null());
    } else {
        cmd.stdout(Stdio::piped()).stderr(Stdio::piped());
    }

    // let child = if *quiet {
    //     std::fs::create_dir_all(log_dir())?;
    //     let log = File::create(log_dir().join("cl.log"))?;
    //     cmd.stdout(Stdio::from(log.try_clone()?))
    //         .stderr(Stdio::from(log))
    //         .spawn()?
    // } else {
    //     cmd.stdout(Stdio::inherit())
    //         .stderr(Stdio::inherit())
    //         .spawn()?
    // };

    Ok(cmd.spawn()?)
}

pub async fn start_nodes(el: &mut Child, cl: &mut Child, quiet: bool) -> anyhow::Result<()> {
    if !quiet {
        if let Some(stdout) = el.stdout.take() {
            tokio::spawn(stream_logs("EL", "\x1b[32m", stdout));
        }
        if let Some(stdout) = cl.stdout.take() {
            tokio::spawn(stream_logs("CL", "\x1b[34m", stdout));
        }
    }

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

async fn stream_logs(prefix: &str, color: &str, stdout: ChildStdout) {
    let mut reader = BufReader::new(stdout).lines();

    const RESET: &str = "\x1b[0m";

    while let Ok(Some(line)) = reader.next_line().await {
        println!("{}[{}]{} {}", color, prefix, RESET, line);
    }
}
