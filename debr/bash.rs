use std::process::{Command, Stdio};
use std::env;
use std::io::{self, BufRead};
use std::ffi::OsStr;
use std::path::Path;
use std::fs;

use colored::*;

pub fn run_cmd<I, S>(executable: S, args: I, working_dir: Option<&Path>) -> io::Result<()>
where
    I: IntoIterator<Item = S>,
    S: AsRef<OsStr>,  // Ensure S implements AsRef<OsStr>
{
    // If working_dir is provided, ensure it exists (create it if necessary)
    if let Some(dir) = working_dir {
        if !dir.exists() {
            println!("Directory does not exist. Creating directory: {}", dir.display());
            fs::create_dir_all(dir)?; // Create the directory recursively
        }
    }

    let mut command = Command::new(executable);
    
    // Set the working directory if provided
    if let Some(dir) = working_dir {
        command.current_dir(dir);
    }
    
    // Add the rest of the arguments to the command
    command.args(args)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::piped()); // Capture stderr

    let mut child = command.spawn()?;

    // Stream stderr while the command is running
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
        let code = status.code().unwrap_or(-1); // Get exit code or use -1 if not available
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

    // Pass the executable path as the first argument to `run_cmd`, followed by the script path
    run_cmd("sh", [script_path.to_str().unwrap()], working_dir)
}

pub fn install() {
    match run_script("assets/install_deps.sh",None) {
        Ok(_) => println!("Dependencies installed successfully."),
        Err(e) => {
            eprintln!("Failed to install dependencies: {}", e);
            std::process::exit(1);
        }
    }
}
