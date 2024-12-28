use reqwest::blocking::ClientBuilder;
use std::process::{Command, Stdio};
use std::error::Error;
use std::fs::{create_dir_all, set_permissions, OpenOptions};
use std::path::Path;
use std::os::unix::fs::PermissionsExt;
use std::io::Write;
use std::collections::HashMap;
use std::time::Duration;
use std::thread;

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

        let mut retries = 0;
        let max_retries = 3;
        let retry_interval = Duration::from_secs(5 * 60); // 5 minutes

        loop {
            match client.get(&url).send() {
                Ok(response) => {
                    if !response.status().is_success() {
                        error(&format!(
                            "Failed to fetch key from {}: {}",
                            url,
                            response.status()
                        ));
                        retries += 1;
                    } else {
                        match response.bytes() {
                            Ok(key_data) => {
                                let output_path = keyrings_dir.join(format!("{}.gpg", file_name));
                                if let Err(e) = dearmor_to(&key_data, &output_path) {
                                    error(&format!(
                                        "Failed to dearmor and write key to {}: {}",
                                        output_path.display(),
                                        e
                                    ));
                                } else {
                                    break; // Success, move to the next keyring
                                }
                            }
                            Err(e) => {
                                error(&format!(
                                    "Failed to read response from {}: {}",
                                    url, e
                                ));
                                retries += 1;
                            }
                        }
                    }
                }
                Err(e) => {
                    error(&format!("Failed to send request to {}: {}", url, e));
                    retries += 1;
                }
            }

            if retries >= max_retries {
                error(&format!(
                    "Exceeded maximum retries for URL: {}",
                    url
                ));
                break;
            }

            // Wait before retrying
            thread::sleep(retry_interval);
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