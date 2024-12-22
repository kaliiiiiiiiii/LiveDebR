use std::env;
use std::fs::{copy, remove_file};
use std::path::Path;
mod cfg_parser;
mod json_cfg;
use crate::Args;


pub fn apply(args: &Args, live_dir: &Path) {
    let executable_path = env::current_exe().unwrap();
    let dir = executable_path.parent().unwrap();
    let bootstrap = live_dir.join("config/bootstrap");

    let config_path = json_cfg::find_config_path(&args.config).unwrap_or_else(|| {
        eprintln!("Error: Configuration file '{}' not found", args.config);
        std::process::exit(1);
    });

    // Read the configuration json file
    let config: json_cfg::Config = json_cfg::read_config(&config_path).unwrap_or_else(|err| {
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
    cfg_parser::set("LB_APT", config.apt.as_str(), live_dir.join("config/common").as_path()).unwrap();

    // Add non-free
    let non_free_path = live_dir.join("config/archives/non_free.list.chroot");
    if config.add_non_free {
        copy(dir.join("assets/non_free.list.chroot"), &non_free_path).unwrap();
    } else {
        remove_file(&non_free_path).ok();
    }

    // Chrome configuration
    println!("Chrome enabled: {}", config.chrome);
}
