use std::env;
use std::fs::{copy, create_dir_all, set_permissions, write};
use std::io::{Error, ErrorKind};
use std::path::Path;
use std::collections::{HashMap, HashSet};
use std::os::unix::fs::PermissionsExt;

mod cfg_parser;
mod json_cfg;
mod sign;
mod deboot_opt;
mod hooks;
mod snap;
use crate::lb;

use sign::place_key;

use crate::Args;

pub fn s(_s: &str) -> String {_s.to_string()}

pub fn apply(args: &Args, live_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    // commonly used paths
    let executable_path = env::current_exe()?;
    let dir = executable_path.parent().unwrap();
    let bootstrap = live_dir.join("config/bootstrap");
    let common = live_dir.join("config/common");
    let includes_after_packages = live_dir.join("config/includes.chroot_after_packages/");

    // parsed values
    let mut includes_parsed: HashSet<String> = HashSet::new();
    let mut includes_from_hook_parsed: HashSet<String> = HashSet::new();
    let mut snaps_parsed: HashSet<String> = HashSet::new();
    let mut extras_parsed : Vec<json_cfg::Extra> = Vec::new();
    let mut e_service_parsed: HashSet<String> = HashSet::new();
    let mut d_service_parsed: HashSet<String> = HashSet::new();
    let mut keyrings_parsed: HashMap<String, String> = HashMap::new();

    // read config
    let config_path = Path::new(&args.config);
    if !Path::new(config_path).exists() {
        return Err(Box::new(Error::new(ErrorKind::NotFound, format!("Configuration file '{}' does not exist", config_path.display()))));
    }
    let config: json_cfg::Config = json_cfg::read_config(&config_path)?;

    
    // lb config
    let dist = &config.dist.unwrap_or(s("bookworm"));
    lb::lb(&["config","--distribution", dist], Some(live_dir))?;

    // architecture & apt archive areas
    let arch = &config.arch.unwrap_or(s("amd64"));
    let archive_areas = &config.archive_areas.unwrap_or(s("main contrib non-free non-free-firmware"));
    let paths_to_set = [
        ("LB_ARCHITECTURE", arch),
        ("LB_ARCHIVE_AREAS", archive_areas),
        ("LB_PARENT_ARCHIVE_AREAS",archive_areas)
    ];
    for (key, value) in paths_to_set.iter() {
        cfg_parser::set(key, value, &bootstrap)?;
    }

    // apt
    cfg_parser::set("LB_APT", &config.apt.unwrap_or(s("apt")), &common)?;
    cfg_parser::set("LB_APT_RECOMMENDS", &config.recommends.unwrap_or(true).to_string(), &common)?;

    // debootstrap options
    let include_extras = vec![s("apt-transport-https"),s("ca-certificates,openssl")];
    let deboot_opts_parsed = deboot_opt::parse(&config.de_boot_opts.unwrap_or(s("")), include_extras);
    cfg_parser::set("DEBOOTSTRAP_OPTIONS", &deboot_opts_parsed,&common)?;

    // configure extra apt packages
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

        keyrings_parsed.insert(name.to_string(), key.to_string());
        cfg_parser::add(&repo_src, &archive_include_path)?;
        place_key(key, &key_path)?;
        includes_from_hook_parsed.extend(extra.add);
    };
    
    // keyringer setup
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
        write(&keyrings_path, serde_json::to_string(&keyrings_parsed)?)?;
        set_permissions(&keyringer_path, PermissionsExt::from_mode(0o755))?;
        set_permissions(&keyrings_path, PermissionsExt::from_mode(0o644))?;
        set_permissions(&service_path, PermissionsExt::from_mode(0o644))?;
        set_permissions(&timer_path, PermissionsExt::from_mode(0o644))?;
        e_service_parsed.extend([s("keyringer.service"), s("keyringer.timer")]);
        includes_parsed.insert(s("pkg-config"));
    }
    
    // darkMode - dark theme
    let dark = config.dark_mode.unwrap_or(true);
    if dark{
        hooks::add_hook("1060-config-gnome-settings.hook.chroot", &hooks::gnome_set_dark()?, live_dir, false)?;
    }

    // snap packages
    if let Some(snaps) = config.snaps{
        snaps_parsed.extend(snaps);
    }
    if snaps_parsed.len() != 0 {
        includes_parsed.insert(s("snapd"));

        let snap_temp_path = includes_after_packages.join("var/snap-download-cache");
        create_dir_all(&snap_temp_path)?;
        for package in &snaps_parsed {
            snap::download(package, &arch,&snap_temp_path)?;
        }

        // snapd live boot install from cache service
        let snapd_installer_service_path = includes_after_packages.join("etc/systemd/system/snapt_installer.service");
        create_dir_all(snapd_installer_service_path.parent().unwrap())?;
        copy(dir.join("assets/snapd_installer.service"), &snapd_installer_service_path)?;
        set_permissions(&snapd_installer_service_path, PermissionsExt::from_mode(0o644))?;
        e_service_parsed.insert(s("snapd_installer.service"));
        
        let content = hooks::snap_install_from(&snaps_parsed, "/var/snap-download-cache")?;
        let script_path = snap_temp_path.join("installer.sh");
        write(&script_path, content)?;
        hooks::chmod_x(script_path)?;
    }

    // enabled//disabled services
    if let Some(e_service) = config.e_service {
        e_service_parsed.extend(e_service);
    }
    if let Some(d_service) = config.d_service {
        d_service_parsed.extend(d_service);
    }
    if d_service_parsed.len() != 0 || e_service_parsed.len() != 0{
        let content = hooks::services(&e_service_parsed, &d_service_parsed)?;
        hooks::add_hook("0500-update-default-services-status", &content, live_dir, false)?;
    }
    
    // apt packages to install
    if let Some(include) = config.include{
        includes_parsed.extend(include);
    }
    
    let content = includes_parsed.iter().cloned().collect::<Vec<String>>().join("\n");
    cfg_parser::add(&content, &live_dir.join("config/package-lists/debr_packages.list.chroot"))?;

    if includes_from_hook_parsed.len() != 0 { // mainly used for "extras" keys
        let content = hooks::apt_install(&includes_from_hook_parsed)?;
        hooks::add_hook("0350-install-apt-packages.hook.chroot", &content, live_dir, false)?;
    }

    if let Some(exclude) = config.exclude {
        if exclude.len() != 0   {
            return Err("excludes are currently not implemented".into());
        }
    }
    Ok(())
}