use reqwest::blocking::ClientBuilder;
use std::process::{Command, Stdio};
use std::error::Error;
use std::fs::{create_dir_all, set_permissions, OpenOptions};
use std::path::Path;
use std::os::unix::fs::PermissionsExt;
use std::io::Write;
use std::collections::HashMap;

use crate::{info, error, KEYRINGS_DIR};

pub fn update_keyrings(keyrings: HashMap<String, String>) -> Result<(), Box<dyn Error>> {
    let keyrings_dir = Path::new(KEYRINGS_DIR);

    // Ensure directory exists
    if !keyrings_dir.exists() {
        create_dir_all(keyrings_dir)?;
        set_permissions(keyrings_dir, PermissionsExt::from_mode(0o755))?;
        info(&format!("Created directory: {}", keyrings_dir.display()));
    }

    let client = ClientBuilder::new()
        .https_only(true)
        .build()?;

    for (file_name, url) in keyrings {
  
        if !url.starts_with("https://") {
            error(&format!("Invalid URL (not HTTPS): {}", url));
            continue;
        }

        let response = match client.get(&url).send() {
            Ok(res) => res,
            Err(e) => {
                error(&format!("Failed to send request to {}: {}", url, e));
                continue;
            }
        };

        if !response.status().is_success() {
            error(&format!("Failed to fetch key from {}: {}", url, response.status()));
            continue;
        }

        let key_data = match response.bytes() {
            Ok(data) => data,
            Err(e) => {
                error(&format!("Failed to read response from {}: {}", url, e));
                continue;
            }
        };

        // Dearmor & write
        let output_path = keyrings_dir.join(format!("{}.gpg", file_name));
        if let Err(e) = dearmor_to(&key_data, &output_path) {
            error(&format!("Failed to dearmor and write key to {}: {}", output_path.display(), e));
            continue;
        }
}

    Ok(())
}

pub fn dearmor_to(input: &[u8], output_path: &Path) -> Result<(), Box<dyn Error>> {
    // Start GPG
    let mut child = Command::new("gpg")
        .arg("--yes")
        .arg("--dearmor")
        .stdin(Stdio::piped())  
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()?;

    if let Some(stdin) = &mut child.stdin {
        stdin.write_all(input)?;
    }

    let output = child.wait_with_output()?;

    if !output.status.success() {
        return Err(format!("gpg --dearmor failed with status: {}", output.status).into());
    }

    let do_create = !output_path.exists();
    let mut file = OpenOptions::new()
        .write(true)
        .create(do_create)
        .truncate(true)
        .open(&output_path)?;
    file.write_all(&output.stdout)?;

    info(&format!("Updated keyring: {}", output_path.display()));
    Ok(())
}