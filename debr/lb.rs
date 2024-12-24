use std::io;
use std::path::Path;
use crate::bash;

pub fn lb(args: &[&str], working_dir: Option<&Path>) -> io::Result<()> {
    bash::cmd("lb", args.iter().copied(), working_dir)
}

pub fn build(working_dir: Option<&Path>) -> io::Result<()> {
    lb(&["build"], working_dir)
}

pub fn clean(working_dir: Option<&Path>) -> io::Result<()> {
    lb(&["clean"], working_dir)
}

pub fn config(working_dir: Option<&Path>) -> io::Result<()> {
    lb(&["config"], working_dir)
}
