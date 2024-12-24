use std::process::{Command, Stdio};
use std::error::Error;

pub fn init(name: &str) -> Result<(), Box<dyn Error>> {
    // Enable and start service or timer
    Command::new("systemctl")
        .arg("enable")
        .arg(name)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()?;
    Command::new("systemctl")
        .arg("start")
        .arg(name)
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()?;

    Ok(())
}

pub fn daemon_reload() -> Result<(), Box<dyn Error>>{
    Command::new("systemctl")
            .arg("daemon-reload")
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .spawn()?;
    Ok(())
}