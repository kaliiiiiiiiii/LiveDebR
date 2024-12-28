use std::collections::HashSet;
use std::fs::{create_dir_all,File};
use std::io::{self, Write};
use std::path::Path;

pub fn services(e_service: &HashSet<String>, d_service: &HashSet<String>) -> std::io::Result<String> {
    // https://github.com/nodiscc/debian-live-config/blob/55677bbd1d8fcfe522f090fb0d77bb1e16027f1d/config/hooks/normal/0350-update-default-services-status.hook.chroot
    let mut script = String::new();
    
    script.push_str("#!/bin/bash\n");
    script.push_str("echo \"I: running $0\"\n\n");

    let gen_for_loop = |services: &HashSet<String>, action: &str| {
        if services.is_empty() {
            return String::new();
        }

        let services_str = escape_to_list(&services);
        let loop_action = if action == "Disabling" { "disable" } else { "enable" };
        let systemctl_action = if action == "Disabling" { "stop" } else { "start" };

        let mut loop_script = String::new();
        loop_script.push_str(&format!("for service in {}; do\n", services_str));
        loop_script.push_str(&format!("    echo \"{} $service\"\n", action));
        loop_script.push_str(&format!("    systemctl {} \"$service\" || true\n", loop_action));
        loop_script.push_str(&format!("    systemctl {} \"$service\" || true\n", systemctl_action));
        loop_script.push_str("done\n");

        loop_script
    };

    if !d_service.is_empty() {
        script.push_str(&gen_for_loop(d_service, "Disabling"));
    }
    if !e_service.is_empty() {
        if !d_service.is_empty() { script.push_str("\n"); }  // Add a newline between blocks if both are present
        script.push_str(&gen_for_loop(e_service, "Enabling"));
    }

    Ok(script)
}

pub fn apt_install(packages: &HashSet<String>) -> std::io::Result<String> {
    let mut script = String::new();

    script.push_str("#!/bin/bash\n");
    script.push_str("echo \"I: running $0\"\n\n");
    script.push_str("set -e  # Exit immediately if a command exits with a non-zero status\n");
    script.push_str("mv /tmp/apt-keyrings-cache-debr/*.gpg /etc/apt/keyrings/\n");
    script.push_str("rm -rf /tmp/apt-keyrings-cache-debr/\n");
    script.push_str("apt update\n\n");
    

    let packages_str = escape_to_list(&packages);

    script.push_str(&format!(
        "echo \"Installing packages: {}\"\n",
        packages_str.replace("\"", "")
    ));
    script.push_str(&format!(
        "DEBIAN_FRONTEND=noninteractive apt-get install -y --no-install-recommends {}\n",
        packages_str
    ));
    script.push_str("\n");
    script.push_str("echo \"Packages installed successfully.\"\n");

    Ok(script)
}

pub fn snap_install(packages: &HashSet<String>) -> io::Result<String> {
    let mut script = String::new();

    script.push_str("#!/bin/bash\n");
    script.push_str("echo \"I: running $0\"\n\n");
    script.push_str("set -e  # Exit immediately if a command exits with a non-zero status\n");
    script.push_str("snap refresh\n\n");

    let packages_str = escape_to_list(&packages);

    script.push_str(&format!(
        "echo \"Installing snap packages: {}\"\n",
        packages_str.replace("\"", "")
    ));
    script.push_str(&format!(
        "snap install {} --classic\n\n",
        packages_str
    ));
    script.push_str("echo \"Snap packages installed successfully.\"\n");

    Ok(script)
}

pub fn gnome_set_dark() -> io::Result<String> {
    let mut script = String::new();

    script.push_str("#!/bin/bash\n");
    script.push_str("echo \"I: running $0\"\n\n");
    script.push_str("set -e  # Exit immediately if a command exits with a non-zero status\n");
    script.push_str("gsettings set org.gnome.desktop.interface color-scheme prefer-dark\n\n");
    Ok(script)
}

pub fn add_hook(name:&str, content:&String, live_dir: &Path) -> std::io::Result<()>{
    let service_hook_path = &live_dir.join("config/hooks/normal/").join(name);
        create_dir_all(service_hook_path.parent().unwrap())?;
        let mut service_hook_file = File::create(service_hook_path)?;
        service_hook_file.write_all(content.as_bytes())?;
        return Ok(());
}

fn escape_to_list(set: &HashSet<String>) -> String {
    let escaped = set
        .iter()
        .map(|p| format!("\"{}\"", p.replace("\"", "\\\""))) // Escape quotes
        .collect::<Vec<String>>()
        .join(" ");
    return escaped;
}