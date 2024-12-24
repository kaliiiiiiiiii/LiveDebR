use std::os::unix::process::CommandExt;
use std::error::Error;
use std::fs::{read, write, copy, File,create_dir_all, remove_file, set_permissions};
use std::io::{Read, Write, stdout, stderr};
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::process::{Command, Stdio};
use std::env::current_exe;
use std::collections::HashMap;
use std::io::{self, ErrorKind};

use clap::{Parser, Subcommand};
use serde_json::Value;

mod ringer;
mod  systemctl;

const CONFIG_FILE: &str = "/etc/keyringer/keyrings.json";
pub const KEYRINGS_DIR: &str = "/etc/apt/keyrings";
const BIN_DIR: &str = "/usr/local/bin";
const SERVICE_FILE: &str = "/etc/systemd/system/keyringer.service";
const TIMER_FILE: &str = "/etc/systemd/system/keyringer.timer";

const HELP_ABOUT: &str = r#"Manages keyrings for APT

Configuration:
  Place a configuration file at `/etc/keyringer/keyrings.json` with the following format:

  {
      "microsoft-archive-keyring": "https://packages.microsoft.com/keys/microsoft.asc",
      "google-chrome": "https://dl.google.com/linux/linux_signing_key.pub"
  }
"#;

#[derive(Parser)]
#[command(name = "keyringer")]
#[command(author = "Aurin Aegerter")]
#[command(version = "1.0")]
#[command(about = HELP_ABOUT)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Uninstall keyringer, remove keyrings and keyrings.json
    Uninstall,
}

fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Uninstall) => uninstall()?,
        None => install()?,
    }

    Ok(())
}


fn install() -> Result<(), Box<dyn Error>> {
    // Install executable
    let bin_path = &Path::new(BIN_DIR).join("keyringer");
    let target_bin = &current_exe()?;
    if target_bin != bin_path {
        copy(target_bin.as_path(), bin_path)?;
        set_permissions(bin_path, PermissionsExt::from_mode(0o755))?;
        info(&format!("Installed executable to {}", bin_path.display()));
    }

    let mut files_written = false;

    let service_content = include_bytes!("../keyringer/assets/keyringer.service");
    let timer_content = include_bytes!("../keyringer/assets/keyringer.timer");

    // Check and write service
    let current_service_content = read(SERVICE_FILE).unwrap_or_default();
    if service_content != current_service_content.as_slice() {
        write(SERVICE_FILE, service_content)?;
        set_permissions(SERVICE_FILE, PermissionsExt::from_mode(0o644))?;
        info(&format!("Installed service {}", SERVICE_FILE));
        files_written = true;
    }

    // Check and write timer
    let current_timer_content = read(TIMER_FILE).unwrap_or_default();
    if timer_content != current_timer_content.as_slice() {
        write(TIMER_FILE, timer_content)?;
        set_permissions(TIMER_FILE, PermissionsExt::from_mode(0o644))?;
        info(&format!("Installed timer {}", TIMER_FILE));
        files_written = true;
    }

    // Reload systemd if files were written
    if files_written {systemctl::daemon_reload()?;}

    // Start and enable service and timer if updated
    if service_content != current_service_content.as_slice() {
        systemctl::init("keyringer.service")?;
    }
    if timer_content != current_timer_content.as_slice() {
        systemctl::init("keyringer.timer")?;
    }

    let keyrings = load_cfg(CONFIG_FILE)?;
    ringer::update_keyrings(keyrings)?;
    Ok(())
}

fn uninstall() -> Result<(), Box<dyn Error>> {
    let bin_path = Path::new(BIN_DIR).join("keyringer");
    let keyrings = load_cfg(CONFIG_FILE)?;

    // Remove keyrings and service files
    for (file_name, _) in keyrings {
        let key_path = &Path::new(KEYRINGS_DIR).join(format!("{}.gpg", file_name));
        if key_path.exists() {
            remove_file(key_path)?;
            info(&format!("Removed keyring: {}", key_path.display()));
        }
    }

    if Path::new(SERVICE_FILE).exists() {
        remove_file(SERVICE_FILE)?;
        info(&format!("Removed service file: {}", SERVICE_FILE));
    }
    if Path::new(TIMER_FILE).exists() {
        remove_file(TIMER_FILE)?;
        info(&format!("Removed timer file: {}", TIMER_FILE));
    }

    if Path::new(CONFIG_FILE).exists() {
        remove_file(CONFIG_FILE)?;
        info(&format!("Removed keyrings.json: {}", CONFIG_FILE));
    }

    systemctl::daemon_reload().ok();

    // Detach process for delayed binary removal
    if bin_path.exists() {
        Command::new("sh")
            .arg("-c")
            .arg(format!(
                "sleep 1 && rm -f {}",
                bin_path.to_str().unwrap()
            ))
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .stdin(Stdio::null())
            .uid(0) // Set UID to root
            .gid(0) // Set GID to root
            .spawn()?; // Detached process

        info("Scheduled binary removal.");
    }

    info("Uninstallation complete.");
    Ok(())
}



fn load_cfg(config_path: &str) -> Result<HashMap<String, String>, Box<dyn Error>> {
    let config_path = Path::new(config_path);
    let config_dir = config_path.parent().ok_or("Invalid config path")?;

    // Ensure config dir
    if !config_dir.exists() {
        create_dir_all(config_dir)?;
        info(&format!("Created directory: {}", config_dir.display()));
    }

    // Ensure config file
    if !config_path.exists() {
        let mut file = File::create(config_path)?;
        file.write_all(b"{}")?;
        set_permissions(config_path, PermissionsExt::from_mode(0o644))?;
        info(&format!("Created empty config file: {}", config_path.display()));
    }

    // Read config
    let mut file = File::open(config_path)?;
    let mut data = String::new();
    file.read_to_string(&mut data)?;

    // Try parsing config
    let parsed: Result<Value, serde_json::Error> = serde_json::from_str(&data);

    match parsed {
        Ok(parsed_value) => {
            let mut keyrings = HashMap::new();
            if let Value::Object(map) = parsed_value {
                for (key, value) in map {
                    if let Value::String(url) = value {
                        keyrings.insert(key, url);
                    } else {
                        error(&format!("Invalid URL format for key: {}", key));
                    }
                }
            } else {
                return Err(Box::new(io::Error::new(
                    ErrorKind::InvalidData,
                    format!("Config file at {} does not contain valid JSON objects.", config_path.display())
                )));
            }

            Ok(keyrings)
        }
        Err(e) => {
            let error_msg = format!("Config file at {} does not contain valid JSON objects.", CONFIG_FILE);
            if e.to_string().contains("trailing characters") {
                return Err(Box::new(io::Error::new(ErrorKind::InvalidData, error_msg)));
            } else {
                return Err(e.into());
            }
        }
    }
}


pub fn info(message: &str) {
    let _ = writeln!(stdout(), "INFO: {}", message);
}

pub fn error(message: &str) {
    let _ = writeln!(stderr(), "ERROR: {}", message);
}
