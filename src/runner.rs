use flate2::read::GzDecoder;
use futures_util::StreamExt;
use rand::TryRngCore;
use rand::rand_core::OsRng;
use reqwest::{Url, get};
use serde::Deserialize;
use std::fs::{File, metadata, set_permissions};
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::path::PathBuf;
use std::process::Stdio;
use tar::Archive;
use tokio::{
    io::AsyncWriteExt,
    process::{Child, Command},
    signal,
};

#[derive(Deserialize)]
struct Release {
    tag_name: String,
    assets: Vec<Asset>,
}

#[derive(Deserialize)]
struct Asset {
    name: String,
    browser_download_url: Url,
}

pub async fn download_reth() -> anyhow::Result<()> {
    let client = reqwest::Client::new();

    let release: Release = client
        .get("https://api.github.com/repos/paradigmxyz/reth/releases/latest")
        .header("User-Agent", "ethup")
        .send()
        .await?
        .json()
        .await?;

    let target_os = match std::env::consts::OS {
        "macos" => "apple-darwin",
        "linux" => "unknown-linux-gnu",
        "windows" => "pc-windows-gnu",
        _ => panic!("unsupported OS"),
    };

    let release_name = format!(
        "reth-{}-{}-{}.tar.gz",
        release.tag_name,
        std::env::consts::ARCH,
        target_os
    );

    let download_url = release
        .assets
        .iter()
        .find(|a| a.name == release_name)
        .map(|a| a.browser_download_url.clone())
        .unwrap();

    let tmp_dir = dirs::home_dir().unwrap().join(".ethup/tmp");
    tokio::fs::create_dir_all(&tmp_dir).await?;
    let tar_path = tmp_dir.join("reth.tar.gz");

    let response = get(download_url).await?;
    let mut file = tokio::fs::File::create(&tar_path).await?;

    let mut stream = response.bytes_stream();

    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        file.write_all(&chunk).await?;
    }

    let tar_gz = File::open(tar_path)?;
    let tar = GzDecoder::new(tar_gz);
    let mut archive = Archive::new(tar);

    let bin_dir = bin_dir();
    std::fs::create_dir_all(&bin_dir)?;
    archive.unpack(&bin_dir)?;

    let mut perms = metadata(bin_dir.join("reth"))?.permissions();
    perms.set_mode(0o755);
    set_permissions(bin_dir.join("reth"), perms)?;

    Ok(())
}

pub async fn download_lighthouse() -> anyhow::Result<()> {
    let client = reqwest::Client::new();

    let release: Release = client
        .get("https://api.github.com/repos/sigp/lighthouse/releases/latest")
        .header("User-Agent", "ethup")
        .send()
        .await?
        .json()
        .await?;

    let target_os = match std::env::consts::OS {
        "macos" => "apple-darwin",
        "linux" => "unknown-linux-gnu",
        "windows" => "pc-windows-gnu",
        _ => panic!("unsupported OS"),
    };

    let release_name = format!(
        "lighthouse-{}-{}-{}.tar.gz",
        release.tag_name,
        std::env::consts::ARCH,
        target_os
    );

    let download_url = release
        .assets
        .iter()
        .find(|a| a.name == release_name)
        .map(|a| a.browser_download_url.clone())
        .unwrap();

    let tmp_dir = dirs::home_dir().unwrap().join(".ethup/tmp");
    tokio::fs::create_dir_all(&tmp_dir).await?;
    let tar_path = tmp_dir.join("lighthouse.tar.gz");

    let response = get(download_url).await?;
    let mut file = tokio::fs::File::create(&tar_path).await?;

    let mut stream = response.bytes_stream();

    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        file.write_all(&chunk).await?;
    }

    let tar_gz = File::open(tar_path)?;
    let tar = GzDecoder::new(tar_gz);
    let mut archive = Archive::new(tar);

    let bin_dir = bin_dir();
    std::fs::create_dir_all(&bin_dir)?;
    archive.unpack(&bin_dir)?;

    let mut perms = metadata(bin_dir.join("lighthouse"))?.permissions();
    perms.set_mode(0o755);
    set_permissions(bin_dir.join("lighthouse"), perms)?;

    Ok(())
}

pub fn bin_dir() -> PathBuf {
    dirs::home_dir().unwrap().join(".ethup/bin")
}

pub fn secret_dir() -> PathBuf {
    dirs::home_dir().unwrap().join(".ethup/secrets")
}

async fn create_jwt() -> anyhow::Result<()> {
    let secrets_path = secret_dir();
    tokio::fs::create_dir_all(&secrets_path).await?;

    let mut key = [0u8; 32];
    OsRng.try_fill_bytes(&mut key)?;

    let hex = key.iter().map(|b| format!("{:02x}", b)).collect::<String>();

    let jwt_path = secrets_path.join("jwt.hex");
    tokio::fs::write(jwt_path, hex).await?;

    Ok(())
}

pub async fn ensure_jwt() -> anyhow::Result<PathBuf> {
    let secrets_dir = secret_dir();
    let jwt_path = secrets_dir.join("jwt.hex");
    if !jwt_path.exists() {
        create_jwt().await?;
    }

    Ok(jwt_path)
}

pub fn data_dir() -> PathBuf {
    dirs::home_dir().unwrap().join(".ethup/data")
}

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
