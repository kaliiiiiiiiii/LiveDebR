use std::io;
use std::path::Path;
use std::fs::remove_dir_all;
use std::env::current_dir;
use crate::bash;

pub fn lb(args: &[&str], working_dir: Option<&Path>) -> io::Result<()> {
    bash::cmd("lb", args.iter().copied(), working_dir)
}

pub fn build(working_dir: Option<&Path>) -> io::Result<()> {
    lb(&["build"], working_dir)
}

pub fn clean(working_dir: Option<&Path>, build: Option<bool>) -> io::Result<()> {
    let args = vec!["clean", "--all"];
    lb(&args, working_dir)?;
    
    if !build.is_some_and(|x| x) {
        let config_dir = if let Some(dir) = working_dir {
            dir.to_path_buf()
        } else {
            current_dir()?
        };
        remove_dir_all(config_dir.join("config/")).ok();
        Ok(())
    } else {
        Ok(())
    }
}
