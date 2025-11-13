use crate::config::{ClConfig, ElConfig};
use crate::layout::{bin_dir, data_dir, secret_dir};

pub fn hoodi_config() -> (ElConfig, ClConfig) {
    let bin = bin_dir();
    let data = data_dir();
    let secrets = secret_dir();
    let jwt = secrets.join("jwt.hex");

    let el = ElConfig {
        name: "reth".to_string(),
        bin: bin.join("reth"),
        data_dir: data.join("reth-hoodi"),
        chain: "hoodi".to_string(),
        http_addr: "127.0.0.1".into(),
        http_port: 8545,
        authrpc_addr: "127.0.0.1".into(),
        authrpc_port: 8551,
        jwt_path: jwt.clone(),
    };

    let cl = ClConfig {
        name: "lighthouse".to_string(),
        bin: bin.join("lighthouse"),
        data_dir: data.join("lighthouse-hoodi"),
        chain: "hoodi".to_string(),
        http_addr: "127.0.0.1".into(),
        http_port: 5052,
        execution_endpoint: el.authrpc_url(),
        execution_jwt: jwt,
        checkpoint_sync_url: Some("https://checkpoint-sync.hoodi.ethpandaops.io".to_string()),
    };

	(el, cl)
}

pub fn _mainnet_config() -> (ElConfig, ClConfig) {
	todo!()
}