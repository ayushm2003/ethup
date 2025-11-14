use std::path::PathBuf;

pub fn bin_dir() -> PathBuf {
    dirs::home_dir().unwrap().join(".ethup/bin")
}

pub fn secret_dir() -> PathBuf {
    dirs::home_dir().unwrap().join(".ethup/secrets")
}

pub fn data_dir() -> PathBuf {
    dirs::home_dir().unwrap().join(".ethup/data")
}

pub fn log_dir() -> PathBuf {
	dirs::home_dir().unwrap().join(".ethup/logs")
}