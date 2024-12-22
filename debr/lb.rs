use std::io;
use std::path::Path;

use crate::bash;

pub fn build(working_dir: Option<&Path>) -> io::Result<()> {
    bash::run_cmd("lb", ["build"], working_dir)
}

pub fn clean(working_dir: Option<&Path>) -> io::Result<()> {
    bash::run_cmd("lb", ["clean"], working_dir)
}

pub fn config(working_dir: Option<&Path>) -> io::Result<()> {
    bash::run_cmd("lb", ["config"], working_dir)
}
