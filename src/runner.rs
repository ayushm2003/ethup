use flate2::read::GzDecoder;
use futures_util::StreamExt;
use reqwest::{Url, get};
use serde::Deserialize;
use std::fs::{File, metadata, set_permissions};
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use tar::Archive;
use tokio::io::AsyncWriteExt;

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
