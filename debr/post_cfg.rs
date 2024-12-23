use std::env;
use std::fs::copy;
use std::path::Path;
mod cfg_parser;
mod json_cfg;
mod sign;
use crate::Args;

pub fn s(_s: &str) -> String {_s.to_string()}

pub fn apply(args: &Args, live_dir: &Path) {
    let executable_path = env::current_exe().unwrap();
    let dir = executable_path.parent().unwrap();
    let bootstrap = live_dir.join("config/bootstrap");
    let common = live_dir.join("config/common");

    let config_path = json_cfg::find_config_path(&args.config).unwrap_or_else(|| {
        eprintln!("Error: Configuration file '{}' not found", args.config);
        std::process::exit(1);
    });

    // Read the configuration json file
    let mut config: json_cfg::Config = json_cfg::read_config(&config_path).unwrap_or_else(|err| {
        eprintln!(
            "Error: Failed to read configuration file '{}': {}",
            config_path.display(),
            err
        );
        std::process::exit(1);
    });

    let dist = config.dist.as_str();

    let paths_to_set = [
        ("LB_DISTRIBUTION", dist),
        ("LB_DISTRIBUTION_CHROOT", dist),
        ("LB_PARENT_DISTRIBUTION_CHROOT", dist),
        ("LB_DISTRIBUTION_BINARY", dist),
        ("LB_PARENT_DISTRIBUTION_BINARY", dist),
        ("LB_ARCHITECTURE", config.arch.as_str()),
    ];

    for (key, value) in paths_to_set.iter() {
        cfg_parser::set(key, value, bootstrap.as_path()).unwrap();
    }

    // APT configuration
    cfg_parser::set("LB_APT", config.apt.as_str(), common.as_path()).unwrap();

    // Add non-free
    let non_free_path = live_dir.join("config/archives/non_free.list.chroot");
    if config.add_non_free {
        copy(dir.join(format!("assets/non_free_list/{}.non_free.list.chroot",dist)), &non_free_path)
        .expect(format!("non_free.list.chroot not available for distribution: {}",dist).as_str());
    }

    if config.chrome {
        sign::place_key("https://dl.google.com/linux/linux_signing_key.pub", live_dir.join("config/archives/google-chrome.key.chroot").as_path()).unwrap();
        
        // Add the repository configuration
        let content = format!("deb http://dl.google.com/linux/chrome/deb/ stable main");
        cfg_parser::add(content.as_str(), live_dir.join("config/archives/google-chrome.list.chroot").as_path()).unwrap();
    
        // Add to include list
        config.include.insert(s("google-chrome-stable"));
    }

    if config.gnome{
        config.include.insert(s("task-gnome-desktop"));
        config.include.insert(s("debian-installer-launcher"));
        cfg_parser::add("! Packages Priority standard", live_dir.join("config/package-lists/standard.list.chroot").as_path()).unwrap();
    }

    let content = config.include.iter().cloned().collect::<Vec<String>>().join("\n");
    cfg_parser::add(content.as_str(), live_dir.join("config/package-lists/installer.list.chroot").as_path()).unwrap();
}
