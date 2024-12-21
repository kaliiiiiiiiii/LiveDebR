use serde::Deserialize;
use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub struct Config {
    pub chrome: bool,
}

pub fn read_config<P: AsRef<Path>>(path: P) -> Result<Config, Box<dyn Error>> {
    println!("Parsing config from {}", path.as_ref().display());

    if !path.as_ref().exists() {
        return Err(format!("Config file '{}' not found", path.as_ref().display()).into());
    }

    let file = File::open(&path)?;
    let reader = BufReader::new(file);

    let config = serde_json::from_reader(reader)?;
    Ok(config)
}

pub fn find_config_path(config_path: &str) -> Option<PathBuf> {
    let path = Path::new(config_path);

    if path.exists() {
        Some(path.to_path_buf())
    } else {
        let exe_dir = std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|dir| dir.to_path_buf()));

        exe_dir.and_then(|dir| {
            let fallback_path = dir.join(config_path);
            if fallback_path.exists() {
                Some(fallback_path)
            } else {
                None
            }
        })
    }
}