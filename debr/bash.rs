use std::process::{Command, Stdio};
use std::env;
use std::io::{self, BufRead};
use colored::Colorize;
use std::path::Path;

pub fn run_script<P: AsRef<Path>>(path: P) -> io::Result<()> {
    let executable_path = env::current_exe()?;
    let script_path = executable_path
        .parent()
        .expect("Executable has no parent directory")
        .join(path);

    let mut child = Command::new("sh")
        .arg(script_path)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::piped())  // Capture stderr
        .spawn()?;

    // Stream stderr while the script is running
    if let Some(stderr) = child.stderr.take() {
        let reader = io::BufReader::new(stderr);
        for line in reader.lines() {
            match line {
                Ok(line) => eprintln!("{}", line.red()),  // Print stderr in red in real-time
                Err(_) => break,
            }
        }
    }

    let status = child.wait()?;

    if !status.success() {
        eprintln!("Error: Script execution failed with {}", status.to_string());
        return Err(io::Error::new(io::ErrorKind::Other, "Script execution failed"));
    }

    Ok(())
}

pub fn install() {
    match run_script("assets/install_deps.sh") {
        Ok(_) => println!("Dependencies installed successfully."),
        Err(e) => {
            eprintln!("Failed to install dependencies: {}", e);
            std::process::exit(1);
        }
    }
}
