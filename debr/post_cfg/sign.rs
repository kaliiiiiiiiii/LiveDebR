use std::fs::{File, create_dir_all};
use std::io::{self, Write};
use std::path::Path;
use reqwest;
use tempfile;
use crate::bash;

/// Download and dearmor key
pub fn place_key(url: &str, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let response = reqwest::blocking::get(url)?;
    if !response.status().is_success() {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            format!("Failed to download {}", url),
        )
        .into());
    }

    let key_data = response.bytes()?;
    let temp_dir = tempfile::tempdir()?;
    let temp_path = temp_dir.path().join("temp_key.pub");
    let mut temp_file = File::create(&temp_path)?;
    temp_file.write_all(&key_data)?;

    if let Some(parent) = path.parent() {
        create_dir_all(parent)?;
    }

    bash::run_cmd("gpg", ["--yes","--dearmor","-o",path.to_str().unwrap(),temp_path.to_str().unwrap()], None).unwrap();

    Ok(())
}
