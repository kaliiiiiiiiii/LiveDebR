use std::env;
use std::fs::{copy, create_dir_all, set_permissions, write, File};
use std::io::{Error, ErrorKind, Write};
use std::path::Path;
use std::collections::{HashMap, HashSet};
use std::os::unix::fs::PermissionsExt;

mod cfg_parser;
mod json_cfg;
mod sign;
mod deboot_opt;
use crate::lb;

use sign::place_key;

use crate::Args;

pub fn s(_s: &str) -> String {_s.to_string()}

pub fn apply(args: &Args, live_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let executable_path = env::current_exe()?;
    let dir = executable_path.parent().unwrap();
    let bootstrap = live_dir.join("config/bootstrap");
    let common = live_dir.join("config/common");
    let includes_after_packages = live_dir.join("config/includes.chroot_after_packages/");

    let mut includes_parsed: HashSet<String> = HashSet::new();
    let mut includes_from_hook_parsed: HashSet<String> = HashSet::new();
    let mut extras_parsed : Vec<json_cfg::Extra> = Vec::new();
    let mut e_service_parsed: HashSet<String> = HashSet::new();
    let mut d_service_parsed: HashSet<String> = HashSet::new();

    let config_path = Path::new(&args.config);
    if !Path::new(config_path).exists() {
        return Err(Box::new(Error::new(ErrorKind::NotFound, format!("Configuration file '{}' does not exist", config_path.display()))));
    }
    let config: json_cfg::Config = json_cfg::read_config(&config_path)?;
    let dist = &config.dist.unwrap_or(s("bookworm"));
    let arch = &config.arch.unwrap_or(s("amd64"));
    let archive_areas = &config.archive_areas.unwrap_or(s("main contrib non-free non-free-firmware"));
    
    lb::lb(&["config","--distribution", dist], Some(live_dir))?;

    

    let paths_to_set = [
        ("LB_ARCHITECTURE", arch),
        ("LB_ARCHIVE_AREAS", archive_areas),
        ("LB_PARENT_ARCHIVE_AREAS",archive_areas)
    ];
    for (key, value) in paths_to_set.iter() {
        cfg_parser::set(key, value, &bootstrap)?;
    }

    cfg_parser::set("LB_APT", &config.apt.unwrap_or(s("apt")), &common)?;
    cfg_parser::set("LB_APT_RECOMMENDS", &config.recommends.unwrap_or(true).to_string(), &common)?;

    let include_extras = vec![s("apt-transport-https"),s("ca-certificates,openssl")];
    let deboot_opts_parsed = deboot_opt::parse(&config.de_boot_opts.unwrap_or(s("")), include_extras);
    cfg_parser::set("DEBOOTSTRAP_OPTIONS", &deboot_opts_parsed,&common)?;

    let mut keyrings: HashMap<String, String> = HashMap::new();
    if let Some(extras) = config.extras{
        extras_parsed.extend_from_slice(&extras);
    }

    for extra in extras_parsed{
        let name = &extra.name;
        let key = &extra.key;
        let repo_src = format!(
            "deb [arch={} signed-by=/etc/apt/keyrings/{}.gpg] {}",
            arch,name, extra.src
        );
        let archive_include_path = includes_after_packages.join(format!("etc/apt/sources.list.d/{}.list", name));
        let key_path = includes_after_packages.join(format!("tmp/apt-keyrings-cache-debr/{}.gpg", name));

        keyrings.insert(name.to_string(), key.to_string());
        cfg_parser::add(&repo_src, &archive_include_path)?;
        place_key(key, &key_path)?;
        includes_from_hook_parsed.extend(extra.add);
    };
    
    if config.keyringer.unwrap_or(true){
        let keyringer_path = includes_after_packages.join("usr/local/bin/keyringer");
        let keyrings_path = includes_after_packages.join("etc/keyringer/keyrings.json");
        let service_path = includes_after_packages.join("etc/systemd/system/keyringer.service");
        let timer_path = includes_after_packages.join("etc/systemd/system/keyringer.timer");
        create_dir_all(keyringer_path.parent().unwrap())?;
        create_dir_all(keyrings_path.parent().unwrap())?;
        create_dir_all(service_path.parent().unwrap())?;
        create_dir_all(timer_path.parent().unwrap())?;
        copy(dir.join("assets/keyringer/keyringer"), &keyringer_path)?;
        copy(dir.join("assets/keyringer/assets/keyringer.service"), &service_path)?;
        copy(dir.join("assets/keyringer/assets/keyringer.timer"), &timer_path)?;
        write(&keyrings_path, serde_json::to_string(&keyrings)?)?;
        set_permissions(&keyringer_path, PermissionsExt::from_mode(0o755))?;
        set_permissions(&keyrings_path, PermissionsExt::from_mode(0o644))?;
        set_permissions(&service_path, PermissionsExt::from_mode(0o644))?;
        set_permissions(&timer_path, PermissionsExt::from_mode(0o644))?;
        e_service_parsed.extend([s("keyringer.service"), s("keyringer.timer")]);
        includes_parsed.insert(s("pkg-config"));
    }
    if let Some(e_service) = config.e_service {
        e_service_parsed.extend(e_service);
    }
    if let Some(d_service) = config.d_service {
        d_service_parsed.extend(d_service);
    }

    if d_service_parsed.len() != 0 || e_service_parsed.len() != 0{
        let content = gen_services_hook(&e_service_parsed, &d_service_parsed)?;
        add_hook("0500-update-default-services-status.hook.chroot", &content, live_dir)?;
    }

    if includes_from_hook_parsed.len() != 0 {
        let content = gen_apt_install_hook(&includes_from_hook_parsed)?;
        add_hook("0350-install-apt-packages.hook.chroot", &content, live_dir)?;
    }
    

    if let Some(include) = config.include{
        includes_parsed.extend(include);
    }
    let content = includes_parsed.iter().cloned().collect::<Vec<String>>().join("\n");
    cfg_parser::add(&content, &live_dir.join("config/package-lists/installer.list.chroot"))?;

    if let Some(exclude) = config.exclude {
        if exclude.len() != 0   {
            return Err("excludes are currently not implemented".into());
        }
    }
    Ok(())
}

pub fn gen_services_hook(e_service: &HashSet<String>, d_service: &HashSet<String>) -> std::io::Result<String> {
    // https://github.com/nodiscc/debian-live-config/blob/55677bbd1d8fcfe522f090fb0d77bb1e16027f1d/config/hooks/normal/0350-update-default-services-status.hook.chroot
    let mut script = String::new();
    
    script.push_str("#!/bin/bash\n");
    script.push_str("echo \"I: running $0\"\n\n");

    let generate_for_loop = |services: &HashSet<String>, action: &str| {
        if services.is_empty() {
            return String::new();
        }

        let services_str = services
            .iter()
            .map(|s| format!("\"{}\"", s.replace("\"", "\\\""))) // Escape quotes
            .collect::<Vec<String>>()
            .join(" ");
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
        script.push_str(&generate_for_loop(d_service, "Disabling"));
    }
    if !e_service.is_empty() {
        if !d_service.is_empty() { script.push_str("\n"); }  // Add a newline between blocks if both are present
        script.push_str(&generate_for_loop(e_service, "Enabling"));
    }

    Ok(script)
}

pub fn gen_apt_install_hook(packages: &HashSet<String>) -> std::io::Result<String> {
    let mut script = String::new();

    script.push_str("#!/bin/bash\n");
    script.push_str("echo \"I: running $0\"\n\n");
    script.push_str("set -e  # Exit immediately if a command exits with a non-zero status\n");
    script.push_str("mv /tmp/apt-keyrings-cache-debr/*.gpg /etc/apt/keyrings/\n");
    script.push_str("rm -rf /tmp/apt-keyrings-cache-debr/\n");
    script.push_str("apt update\n\n");
    

    let packages_str = packages
        .iter()
        .map(|p| format!("\"{}\"", p.replace("\"", "\\\""))) // Escape quotes
        .collect::<Vec<String>>()
        .join(" ");

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

fn add_hook(name:&str, content:&String, live_dir: &Path) -> std::io::Result<()>{
    let service_hook_path = &live_dir.join("config/hooks/normal/").join(name);
        create_dir_all(service_hook_path.parent().unwrap())?;
        let mut service_hook_file = File::create(service_hook_path)?;
        service_hook_file.write_all(content.as_bytes())?;
        return Ok(());
}