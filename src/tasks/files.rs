use serde::{Deserialize, Serialize};
use std::fs;
use std::os::unix::fs as unix_fs;
use std::path::{Path, PathBuf};

#[derive(Serialize, Deserialize, Debug)]
pub struct CopyPathConfig {
    pub source: PathBuf,
    pub symlink: bool,
}

pub fn copy_path(
    copy_config: &CopyPathConfig,
    source_dir: &Path,
    target_dir: &Path,
) -> Result<(), std::io::Error> {
    let source_entry = source_dir.join(&copy_config.source);
    let target_entry = target_dir.join(&copy_config.source);

    if copy_config.symlink {
        unix_fs::symlink(source_entry, target_entry)?;
    } else {
        fs::copy(source_entry, target_entry)?;
    }

    Ok(())
}
