use std::fs::{create_dir_all, OpenOptions};
use std::process::{Command, Stdio};
use std::io::{self, Write};
use std::path::Path;
use reqwest::blocking::ClientBuilder;

pub fn place_key(url: &str, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let client = ClientBuilder::new()
        .https_only(true)
        .build()?;

    if !url.starts_with("https://") {
        return Err(io::Error::new(
            io::ErrorKind::Unsupported,
            format!("https is supported only, url: {}", url),
        )
        .into());
    }
    let response = client.get(url).send()?;

    if !response.status().is_success() {
        return Err(io::Error::new(
            io::ErrorKind::Other,
            format!("Failed to download {}", url),
        )
        .into());
    }

    let key_data = response.bytes()?;

    // Ensure the parent directory exists
    if let Some(parent) = path.parent() {
        create_dir_all(parent)?;
    }

    // Call dearmor_to instead of running the bash command directly
    dearmor_to(&key_data, path)?;

    Ok(())
}

fn dearmor_to(input: &[u8], output_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    // Start gpg
    let mut child = Command::new("gpg")
        .arg("--yes")
        .arg("--dearmor")
        .stdin(Stdio::piped())  // Pipe input
        .stdout(Stdio::piped()) // Capture output
        .spawn()?;

    // Write data
    if let Some(stdin) = &mut child.stdin {
        stdin.write_all(input)?;
    }

    // Capture output
    let output = child.wait_with_output()?;

    if !output.status.success() {
        eprintln!("GPG dearmor failed: {}", output.status);
        return Err(format!("gpg --dearmor failed with status: {}", output.status).into());
    }
    let do_create = !output_path.exists();
    let mut file = OpenOptions::new()
        .write(true)
        .create(do_create)
        .truncate(true)
        .open(&output_path)?;
    file.write_all(&output.stdout)?;
    Ok(())
}