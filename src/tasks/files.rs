use color_eyre::eyre::Context;
use serde::{Deserialize, Serialize};
use std::fs;
use std::os::unix::fs as unix_fs;
use std::path::{Path, PathBuf};

#[derive(Serialize, Deserialize, Debug)]
pub struct CopyPathConfig {
    pub source: PathBuf,
    #[serde(default)]
    pub symlink: bool,
}

pub fn copy_path(
    copy_config: &CopyPathConfig,
    source_dir: &Path,
    target_dir: &Path,
) -> color_eyre::Result<()> {
    let source_entry = source_dir.join(&copy_config.source);
    let target_entry = target_dir.join(&copy_config.source);

    println!(
        "Copying {} to {}",
        source_entry.display(),
        target_entry.display(),
    );

    if copy_config.symlink {
        unix_fs::symlink(&source_entry, &target_entry).context(format!(
            "failed to symlink {} to {}",
            source_entry.display(),
            target_entry.display(),
        ))?;
    } else {
        fs::copy(&source_entry, &target_entry).context(format!(
            "failed to copy {} to {}",
            source_entry.display(),
            target_entry.display(),
        ))?;
    }

    Ok(())
}
