use std::path::PathBuf;

pub struct ElConfig {
    pub name: String,
    pub bin: PathBuf,
    pub chain: String,
    pub data_dir: PathBuf,
    pub http_addr: String,
    pub http_port: u16,
    pub authrpc_addr: String,
    pub authrpc_port: u16,
    pub jwt_path: PathBuf,
}

impl ElConfig {
	pub fn rpc_url(&self) -> String {
		format!("http://{}:{}", self.http_addr, self.http_port)
	}

	pub fn authrpc_url(&self) -> String {
		format!("http://{}:{}", self.authrpc_addr, self.authrpc_port)
	}
}

pub struct ClConfig {
    pub name: String,
    pub bin: PathBuf,
	pub data_dir: PathBuf,
    pub chain: String,
    pub http_addr: String,
    pub http_port: u16,
    pub execution_endpoint: String,
    pub execution_jwt: PathBuf,
    pub checkpoint_sync_url: Option<String>,
}

impl ClConfig {
    pub fn http_url(&self) -> String {
        format!("http://{}:{}", self.http_addr, self.http_port)
    }
}