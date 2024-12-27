use std::env;
use std::fs::{copy, create_dir_all, set_permissions, metadata, write, File};
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
    let includes_chroot = live_dir.join("config/includes.chroot_before_packages/");

    let mut includes_parsed: HashSet<String> = HashSet::new();
    let mut extras_parsed : Vec<json_cfg::Extra> = Vec::new();
    let mut extra_e_service: HashSet<String> = HashSet::new();
    let mut extra_d_service: HashSet<String> = HashSet::new();

    let config_path = Path::new(&args.config);
    if !Path::new(config_path).exists() {
        return Err(Box::new(Error::new(ErrorKind::NotFound, format!("Configuration file '{}' does not exist", config_path.display()))));
    }
    let config: json_cfg::Config = json_cfg::read_config(&config_path)?;
    let dist = &config.dist.unwrap_or(s("bookworm"));
    lb::lb(&["config","--distribution", dist], Some(live_dir))?;

    let archive_areas = config.archive_areas.unwrap_or(s("main contrib non-free non-free-firmware"));
    

    let paths_to_set = [
        ("LB_ARCHITECTURE", &config.arch.unwrap_or(s("amd64"))),
        ("LB_ARCHIVE_AREAS", &archive_areas),
        ("LB_PARENT_ARCHIVE_AREAS",&archive_areas)
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
        let archive_path = live_dir.join(format!("config/archives/{}.list.chroot",name));
        let key_path = live_dir.join(format!("config/archives/{}.key.chroot",name));

        keyrings.insert(name.to_string(), key.to_string());
        cfg_parser::add(&extra.src, &archive_path)?;
        place_key(key, &key_path)?;
        includes_parsed.extend(extra.add);
    };
    
    if config.keyringer.unwrap_or(true){
        let keyringer_path = includes_chroot.join("usr/local/bin/keyringer");
        let keyrings_path = includes_chroot.join("etc/keyringer/keyrings.json");
        let service_path = includes_chroot.join("etc/systemd/system/keyringer.service");
        let timer_path = includes_chroot.join("etc/systemd/system/keyringer.timer");
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
        extra_e_service.extend([s("keyringer.service"), s("keyringer.timer")]);
        includes_parsed.insert(s("pkg-config"));
    }
    if let Some(e_service) = config.e_service {
        extra_e_service.extend(e_service);
    }
    if let Some(d_service) = config.d_service {
        extra_d_service.extend(d_service);
    }

    if extra_d_service.len() != 0 || extra_e_service.len() != 0{
        let service_hook_path = &live_dir.join("config/hooks/normal/0350-update-default-services-status.hook.chroot");
        create_dir_all(service_hook_path.parent().unwrap())?;
        let mut service_hook_file = File::create(service_hook_path)?;
        service_hook_file.write_all(gen_services_hook(&extra_e_service, &extra_d_service)?.as_bytes())?;
        chmod_x(service_hook_path)?;
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
            return String::new()
        }

        let services_str = services.iter().cloned().collect::<Vec<String>>().join(" ");
        let loop_action = if action == "Disabling" { "disable" } else { "enable" };
        let systemctl_action = if action == "Disabling" { "stop" } else { "start" };

        let mut loop_script = String::new();
        loop_script.push_str(&format!("for service in \"{}\"; do\n", services_str));
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

fn chmod_x<P: AsRef<Path>>(path: P) -> std::io::Result<()> {
    let metadata = metadata(&path)?;
    let mut permissions = metadata.permissions();
    permissions.set_mode(permissions.mode() | 0o111);
    set_permissions(path, permissions)
}