use clap::{Parser, Subcommand};
use std::os::unix::process::CommandExt;
use std::error::Error;
use std::fs::{create_dir_all, remove_file, set_permissions};
use std::fs;
use std::io::{Read, Write, stdout, stderr};
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::process::{Command, Stdio};
use std::env::current_exe;
use std::collections::HashMap;
use reqwest::blocking::ClientBuilder;
use serde_json::{Value, json};

const CONFIG_FILE: &str = "/etc/keyringer/keyrings.json";
const KEYRINGS_DIR: &str = "/etc/apt/keyrings";
const BIN_DIR: &str = "/usr/local/bin";
const SERVICE_FILE: &str = "/etc/systemd/system/keyringer.service";
const TIMER_FILE: &str = "/etc/systemd/system/keyringer.timer";

#[derive(Parser)]
#[command(name = "keyringer")]
#[command(author = "Aurin Aegerter")]
#[command(version = "1.0")]
#[command(about = "Manage keyrings for APT")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Uninstall keyringer and remove keyrings
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

    daemon_reload().ok();

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


fn install() -> Result<(), Box<dyn Error>> {
    // Install executable
    let bin_path = &Path::new(BIN_DIR).join("keyringer");
    let target_bin = &current_exe()?;
    if target_bin != bin_path {
        fs::copy(target_bin.as_path(), bin_path)?;
        set_permissions(bin_path, PermissionsExt::from_mode(0o755))?;
        info(&format!("Installed executable to {}", bin_path.as_path().display()));
    }

    let mut files_written = false;

    let service_content = include_bytes!("../keyringer/assets/keyringer.service");
    let timer_content = include_bytes!("../keyringer/assets/keyringer.timer");

    // Check and write service
    let current_service_content = fs::read(SERVICE_FILE).unwrap_or_default();
    if service_content != current_service_content.as_slice() {
        fs::write(SERVICE_FILE, service_content)?;
        set_permissions(SERVICE_FILE, PermissionsExt::from_mode(0o644))?;
        info(&format!("Installed service {}", SERVICE_FILE));
        files_written = true;
    }

    // Check and write timer
    let current_timer_content = fs::read(TIMER_FILE).unwrap_or_default();
    if timer_content != current_timer_content.as_slice() {
        fs::write(TIMER_FILE, timer_content)?;
        set_permissions(TIMER_FILE, PermissionsExt::from_mode(0o644))?;
        info(&format!("Installed timer {}", TIMER_FILE));
        files_written = true;
    }

    // Reload systemd if files were written
    if files_written {daemon_reload()?;}

    // Start and enable service and timer if updated
    if service_content != current_service_content.as_slice() {
        init("keyringer.service")?;
    }
    if timer_content != current_timer_content.as_slice() {
        init("keyringer.timer")?;
    }

    update_keyrings()?;
    Ok(())
}

fn update_keyrings() -> Result<(), Box<dyn Error>> {
    let keyrings = load_cfg(CONFIG_FILE)?;
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
        // Validate URL
        if !url.starts_with("https://") {
            error(&format!("Invalid URL (not HTTPS): {}", url));
            continue;
        }

        // Fetch key
        let response = client.get(&url).send()?;
        if !response.status().is_success() {
            error(&format!("Failed to fetch key from {}: {}", url, response.status()));
            continue;
        }

        // Get raw key
        let key_data = response.bytes()?;

        // Dearmor & write
        let output_path = keyrings_dir.join(format!("{}.gpg", file_name));
        if let Err(e) = dearmor_to(&key_data, &output_path) {
            error(&format!("Failed to dearmor key for {}: {}", file_name, e));
        }
    }

    Ok(())
}

fn dearmor_to(input: &[u8], output_path: &Path) -> Result<(), Box<dyn Error>> {
    // Start GPG
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
        error(&format!("GPG dearmor failed: {}", output.status));
        return Err(format!("gpg --dearmor failed with status: {}", output.status).into());
    }

    // Write output
    let mut file = fs::File::create(output_path)?;
    file.write_all(&output.stdout)?;

    info(&format!("Processed key and saved: {}", output_path.display()));
    Ok(())
}

fn init(name: &str) -> Result<(), Box<dyn Error>> {
    // Enable and start service or timer
    Command::new("systemctl")
        .arg("enable")
        .arg(name)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()?;
    Command::new("systemctl")
        .arg("start")
        .arg(name)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()?;

    Ok(())
}

fn daemon_reload() -> Result<(), Box<dyn Error>>{
    Command::new("systemctl")
            .arg("daemon-reload")
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .spawn()?;
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
        let default_data = json!({});
        let mut file = fs::File::create(config_path)?;
        file.write_all(default_data.to_string().as_bytes())?;
        set_permissions(config_path, PermissionsExt::from_mode(0o644))?;
        info(&format!("Created empty config file: {}", config_path.display()));
    }

    // Read config
    let mut file = fs::File::open(config_path)?;
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
                error(&format!("Config file at {} does not contain valid JSON objects.", config_path.display()));
                return Err("Invalid config format".into());
            }

            Ok(keyrings)
        }
        Err(e) => {
            error(&format!("Failed to parse config file at {}: {}", config_path.display(), e));
            if e.to_string().contains("trailing characters") {
                error("Ensure the JSON config is properly formatted without extra commas or characters.");
            }
            Err(e.into())
        }
    }
}

fn info(message: &str) {
    let _ = writeln!(stdout(), "INFO: {}", message);
}

fn error(message: &str) {
    let _ = writeln!(stderr(), "ERROR: {}", message);
}
