use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::error::Error;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;
use std::env::current_exe;

#[derive(Debug, Deserialize, Serialize, Clone)] // Added Clone here
#[serde(rename_all = "camelCase")]
pub struct Config {
    pub arch: Option<String>,
    pub dist: Option<String>,
    pub archive_areas: Option<String>,
    pub recommends: Option<bool>,
    pub apt: Option<String>,
    pub include: Option<HashSet<String>>,
    pub exclude: Option<HashSet<String>>,
    pub snaps: Option<HashSet<String>>,

    pub extras: Option<Vec<Extra>>,
    pub keyringer: Option<bool>,
    pub dark_mode: Option<bool>,
    pub de_boot_opts: Option<String>,
    pub requires: Option<HashSet<String>>,
    pub e_service: Option<HashSet<String>>,
    pub d_service: Option<HashSet<String>>,

    pub lang: Option<String>,
    
}

#[derive(Debug, Deserialize, Serialize, Clone)] // Added Clone here
#[serde(rename_all = "camelCase")]
pub struct Extra {
    pub name: String,
    pub key: String,
    pub src: String,
    pub add: HashSet<String>,
}

pub fn merge(this_config:&Config, other_config: &Config) -> Result<Config, Box<dyn Error>> {
    let this_json = serde_json::to_value(this_config)?;
    let other_json = serde_json::to_value(&other_config)?;
    let mut this_map = this_json.as_object().unwrap().clone();
    let other_map = other_json.as_object().unwrap();

    for (key, other_value) in other_map {
            if let Some(this_value) = this_json.get(key){
                if other_value.is_null(){}
                else if this_value.is_null(){
                    this_map.insert(key.clone(), other_value.clone());
                }
                else if this_value.is_array() && other_value.is_array(){
                    let mut mut_array = this_value.as_array().unwrap().clone();
                    mut_array.extend_from_slice(other_value.as_array().unwrap());
                    this_map.insert(key.clone(), serde_json::to_value(mut_array)?);
                }else if this_value != other_value {
                    return Err(format!("Conflict in `{}` field\nThisValue:\n{}\nOtherValue:\n{}", key, this_value, other_value).into());
                }
            }else{
                this_map.insert(key.clone(), other_value.clone());
            }
        }
    
    
    let new_value = serde_json::Value::Object(this_map);
    println!("{}\n", new_value.to_string());
    let new_config: Config = serde_json::from_value(new_value)?;

    Ok(new_config)
}

pub fn add(this_config: &Config, path: &Path) -> Result<Config, Box<dyn Error>> {
    let additional_config = read_config(path)?;

    match merge(this_config, &additional_config) {
        Ok(merged) => Ok(merged),
        Err(e) => {
            let conflict_message = format!(
                "Error merging configurations: {}\nWhile merging with file `{}`.",
                e,
                path.display()
            );
            Err(conflict_message.into())
        }
    }
}

pub fn read_config(path: &Path) -> Result<Config, Box<dyn Error>> {
    let mut final_path = path.to_path_buf();
    
    if !path.exists() {
        final_path.set_extension("json");
        final_path = current_exe()?.parent().unwrap().join("assets/modules/").join(final_path);
        if !final_path.exists() {
            return Err(format!("Module {} not found", final_path.display()).into());
        }
        println!("[Config module] {}", path.display());
    } else {
        if !final_path.exists() {
            return Err(format!("Config not found at {}", path.display()).into());
        }
        println!("[Config       ] {}", final_path.display());
    }

    let file = File::open(final_path)?;
    let reader = BufReader::new(file);
    let mut config: Config = serde_json::from_reader(reader)?;

    if let Some(requires) = config.requires.clone() {
        for required_path in requires {
            config = add(&config, Path::new(&required_path))?;
        }
    }
    Ok(config)
}