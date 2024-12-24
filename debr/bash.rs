use std::process::{Command, Stdio};
use std::env;
use std::io::{self, BufRead};
use std::ffi::OsStr;
use std::path::Path;
use std::fs;

use colored::*;

pub fn cmd<I, S>(executable: S, args: I, working_dir: Option<&Path>) -> io::Result<()>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,
{
    let mut command = Command::new(executable);
    if let Some(dir) = working_dir {
        if !dir.exists() {
            println!("Directory does not exist. Creating directory: {}", dir.display());
            fs::create_dir_all(dir)?;
        }
        command.current_dir(dir);
    }
    
    command.args(args)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::piped()); // Capture stderr

    let mut child = command.spawn()?;

    // print stderr in red
    if let Some(stderr) = child.stderr.take() {
        let reader = io::BufReader::new(stderr);
        for line in reader.lines() {
            match line {
                Ok(line) => eprintln!("{}", line.as_str().red()), // Convert to &str to use `red()`
                Err(_) => break,
            }
        }
    }

    let status = child.wait()?;

    if !status.success() {
        let code = status.code().unwrap_or(-1);
        return Err(io::Error::new(
            io::ErrorKind::Other,
            format!("Command execution failed with status code: {}", code),
        ));
    }

    Ok(())
}


pub fn run_script<P: AsRef<Path>>(path: P, working_dir: Option<&Path>) -> io::Result<()> {
    let executable_path = env::current_exe()?;
    let script_path = executable_path
        .parent()
        .expect("Executable has no parent directory")
        .join(path);

    cmd("sh", [script_path.to_str().unwrap()], working_dir)
}

pub fn install() -> io::Result<()> {
    run_script("assets/install_deps.sh", None).map(|_| {
        println!("Dependencies installed successfully.");
    })
}

