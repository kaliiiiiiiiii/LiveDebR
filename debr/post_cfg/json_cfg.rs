use serde::Deserialize;
use std::collections::HashSet;
use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};

pub fn s(_s: &str) -> String {
    _s.to_string()
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    #[serde(default = "Defaults::apt")]
    pub apt: String,
    #[serde(default = "Defaults::dist")]
    pub dist: String,
    #[serde(default = "Defaults::arch")]
    pub arch: String,
    #[serde(default = "Defaults::add_non_free")]
    pub add_non_free: bool,
    #[serde(default = "Defaults::chrome")]
    pub chrome: bool,
    #[serde(default = "Defaults::gnome")]
    pub gnome: bool,
    #[serde(default = "Defaults::lang")]
    pub lang: String,
    #[serde(default = "Defaults::include")]
    pub include: HashSet<String>,
    #[serde(default = "Defaults::exclude")]
    pub exclude: HashSet<String>,
    #[serde(default)]
    pub extras: Vec<ExtraConfig>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExtraConfig {
    pub name: String,
    pub key: String,
    pub src: String,
    pub repo: Vec<String>,
}

pub struct Defaults;
impl Defaults {
    pub fn apt() -> String {s("apt")}
    pub fn dist() -> String {s("bullseye")}
    pub fn arch() -> String {s("amd64")}
    pub fn add_non_free() -> bool {true}
    pub fn chrome() -> bool {true}
    pub fn gnome() -> bool {true}
    pub fn lang() -> String {s("en")}
    pub fn include() -> HashSet<String> {HashSet::new()}
    pub fn exclude() -> HashSet<String> {HashSet::new()}
}

pub fn read_config<P: AsRef<Path>>(path: P) -> Result<Config, Box<dyn Error>> {
    println!("Parsing config from {}", path.as_ref().display());

    if !path.as_ref().exists() {
        return Err(format!("Config file '{}' not found", path.as_ref().display()).into());
    }

    let file = File::open(&path)?;
    let reader = BufReader::new(file);

    let config: Config = serde_json::from_reader(reader)?;
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
