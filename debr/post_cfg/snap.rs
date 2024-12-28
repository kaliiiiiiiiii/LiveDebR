use std::process::{Command, Stdio};
use std::env;
use std::io;
use std::fs::{self, create_dir_all};
use std::path::Path;

pub fn download(package: &str, architecture: &str, target_dir: &Path) -> io::Result<()> {
    env::set_var("UBUNTU_STORE_ARCH", architecture);

    // Ensure the target directory exists
    if !target_dir.exists() {
        create_dir_all(target_dir)?;
    }

    // snap download
    let output = Command::new("snap")
        .arg("download")
        .arg(package)
        .env("UBUNTU_STORE_ARCH", architecture)
        .current_dir(target_dir)
        .stdout(Stdio::inherit()) 
        .stderr(Stdio::inherit()) 
        .stdin(Stdio::inherit())
        .output()?;

    if !output.status.success() {
        let err_msg = String::from_utf8_lossy(&output.stderr);
        return Err(io::Error::new(io::ErrorKind::Other, format!("Failed to download {}: {}", package, err_msg)));
    }

    let assert_file_pattern = format!("{}_", package);
    let snap_file_pattern = format!("{}_", package);

    let mut assert_file = None;
    let mut snap_file = None;

    // find the downloaded files
    for entry in fs::read_dir(target_dir)? {
        let entry = entry?;
        let file_name = entry.file_name().to_string_lossy().to_string();

        // Check for .assert file that matches the prefix
        if file_name.starts_with(&assert_file_pattern) && file_name.ends_with(".assert") {
            let assert_path = entry.path().canonicalize()?;
            assert_file = Some(assert_path);
        }

        // Check for .snap file that matches the prefix
        if file_name.starts_with(&snap_file_pattern) && file_name.ends_with(".snap") {
            let snap_path = entry.path().canonicalize()?;
            snap_file = Some(snap_path);
        }
    }

    // Ensure both files are found
    if let (Some(assert_file), Some(snap_file)) = (assert_file, snap_file) {
        let snap_path = target_dir.join(format!("{}.snap", package));
        let assert_path = target_dir.join(format!("{}.assert", package));

        fs::rename(snap_file, &snap_path)?;
        fs::rename(assert_file, &assert_path)?;

        println!("Downloaded snap: {}", package);
        Ok(())
    } else {
        Err(io::Error::new(io::ErrorKind::NotFound, "Expected .assert or .snap files not found"))
    }
}
