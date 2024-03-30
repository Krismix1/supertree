use serde::{Deserialize, Serialize};
use std::{path::Path, process::Command};

#[derive(Serialize, Deserialize, Debug)]
pub struct ShellConfig {
    pub cmd: String,
}

pub fn run_shell(config: &ShellConfig, work_dir: &Path) -> Result<(), std::io::Error> {
    eprintln!("Running command `{}`", config.cmd);

    Command::new("bash")
        .arg("-c")
        .arg(&config.cmd)
        .current_dir(work_dir)
        .output()?;

    Ok(())
}
