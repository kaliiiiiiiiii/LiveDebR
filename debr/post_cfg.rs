use std::env;
use std::fs::{copy, create_dir_all, set_permissions, write};
use std::io::{ErrorKind, Error};
use std::path::Path;
use std::collections::{HashMap, HashSet};
use std::os::unix::fs::PermissionsExt;

mod cfg_parser;
mod json_cfg;
mod sign;

use sign::place_key;

use crate::Args;

const EXTRA_NAME_BLACKLIST:&str = "non-free";

pub fn s(_s: &str) -> String {_s.to_string()}

pub fn apply(args: &Args, live_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let executable_path = env::current_exe()?;
    let dir = executable_path.parent().unwrap();
    let bootstrap = live_dir.join("config/bootstrap");
    let common = live_dir.join("config/common");

    let config_path = Path::new(&args.config);
    if !Path::new(config_path).exists() {
        return Err(Box::new(Error::new(ErrorKind::NotFound, format!("Configuration file '{}' does not exist", config_path.display()))));
    }
    let mut config: json_cfg::Config = json_cfg::read_config(&config_path)?;
    let dist = &config.dist;

    let paths_to_set = [
        ("LB_DISTRIBUTION", dist),
        ("LB_DISTRIBUTION_CHROOT", dist),
        ("LB_PARENT_DISTRIBUTION_CHROOT", dist),
        ("LB_DISTRIBUTION_BINARY", dist),
        ("LB_PARENT_DISTRIBUTION_BINARY", dist),
        ("LB_ARCHITECTURE", &config.arch),
    ];
    for (key, value) in paths_to_set.iter() {
        cfg_parser::set(key, value, &bootstrap)?;
    }

    cfg_parser::set("LB_APT", &config.apt, &common)?;
    cfg_parser::set("DEBOOTSTRAP_OPTIONS", &config.de_boot_opts,&common)?;
    
    // Add non-free
    let non_free_path = live_dir.join("config/archives/non_free.list.chroot");
    if config.add_non_free {
        copy(dir.join(format!("assets/non_free_list/{}.non_free.list.chroot",dist)), &non_free_path)
        .expect(&format!("non_free.list.chroot not available for distribution: {}",dist));
    }
    if config.chrome {
        let extra = json_cfg::Extra {
            name:s("google-chrome"),
            key:s("https://dl.google.com/linux/linux_signing_key.pub"),
            src:s("deb http://dl.google.com/linux/chrome/deb/ stable main"),
            add: HashSet::from([s("google-chrome-stable")])
        };
        config.extras.push(extra);
    }

    if config.gnome{
        config.include.extend([s("task-gnome-desktop"),s("debian-installer-launcher")]);
        cfg_parser::add("! Packages Priority standard", &live_dir.join("config/package-lists/standard.list.chroot"))?;
    }

    let mut keyrings: HashMap<String, String> = HashMap::new();
    for extra in config.extras{
        let name = &extra.name;
        let key = &extra.key;
        let archive_path = live_dir.join(format!("config/archives/{}.list.chroot",name));
        let key_path = live_dir.join(format!("config/archives/{}.key.chroot",name));
        if EXTRA_NAME_BLACKLIST.to_string().split_whitespace().any(|x| x == name){
            return Err(Box::new(Error::new(
                ErrorKind::AlreadyExists,
                format!("Name already reserved, not allowed for extras {}", name),
            )));
        }
        keyrings.insert(name.to_string(), key.to_string());
        cfg_parser::add(&extra.src, &archive_path)?;
        place_key(key, &key_path)?;
        config.include.extend(extra.add);
    };
    if config.keyringer{
        let keyringer_path = live_dir.join("config/includes/usr/local/bin/keyringer");
        let keyrings_path = live_dir.join("config/includes/etc/keyringer/keyrings.json");
        create_dir_all(keyringer_path.parent().unwrap())?;
        create_dir_all(keyrings_path.parent().unwrap())?;
        copy(dir.join("assets/keyringer"), &keyringer_path)?;
        write(&keyrings_path, serde_json::to_string(&keyrings)?)?;
        set_permissions(&keyringer_path, PermissionsExt::from_mode(0o755))?;
        set_permissions(&keyrings_path, PermissionsExt::from_mode(0o644))?;
    }
    let content = config.include.iter().cloned().collect::<Vec<String>>().join("\n");
    cfg_parser::add(&content, &live_dir.join("config/package-lists/installer.list.chroot"))?;
    Ok(())
}
