use serde::Deserialize;
use std::collections::HashSet;
use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

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
    #[serde(default="Defaults::extras")]
    pub extras: Vec<Extra>,
    #[serde(default="Defaults::keyringer")]
    pub keyringer: bool,
    #[serde(default = "Defaults::de_boot_opts")]
    pub de_boot_opts: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Extra {
    pub name: String,
    pub key: String,
    pub src: String,
    pub add: HashSet<String>,
}

pub struct Defaults;
impl Defaults {
    pub fn apt()-> String {s("apt")}
    pub fn dist() -> String {s("bullseye")}
    pub fn arch() -> String {s("amd64")}
    pub fn add_non_free() -> bool {true}
    pub fn chrome() -> bool {true}
    pub fn gnome() -> bool {true}
    pub fn lang() -> String {s("en")}
    pub fn include() -> HashSet<String> {HashSet::new()}
    pub fn exclude() -> HashSet<String> {HashSet::new()}
    pub fn extras() -> Vec<Extra> {Vec::new()}
    pub fn keyringer() -> bool {true}
    pub fn de_boot_opts() -> String {String::new()}
}

pub fn read_config(path: &Path) -> Result<Config, Box<dyn Error>> {
    let config_path: &Path;
    let current_dir = std::env::current_dir()?;
    let full_path = current_dir.join(path);

    if !path.is_file() {
        config_path = full_path.as_path();
    } else {
        // fallback : relative path
        config_path = path;
    }

    println!("Parsing config from {}", &config_path.canonicalize().unwrap().display());
    if !config_path.is_file() {
        return Err(format!("Config file '{}' not found", path.display()).into());
    }

    let file = File::open(config_path)?;
    let reader = BufReader::new(file);
    let config: Config = serde_json::from_reader(reader)?;
    Ok(config)
}
