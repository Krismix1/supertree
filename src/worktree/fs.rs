use std::error::Error;
use std::fs;
use std::os::unix::fs as unix_fs;
use std::path::Path;

pub struct CopyTargets {
    /// Path relative to a worktree
    path: String,
    /// Whether or not a symlink should be used over a full-copy
    use_symlink: bool,
}

impl CopyTargets {
    pub fn new(source_path: String, use_symlink: bool) -> Self {
        Self {
            path: source_path,
            use_symlink,
        }
    }
}

pub fn copy_files(
    source_dir: &Path,
    target_dir: &Path,
    targets: &[CopyTargets],
) -> Result<(), Box<dyn Error>> {
    for target in targets {
        let source_entry = source_dir.join(target.path.clone());
        let target_entry = target_dir.join(target.path.clone());

        if target.use_symlink {
            unix_fs::symlink(source_entry, target_entry)?;
        } else {
            fs::copy(source_entry, target_entry)?;
        }
    }

    Ok(())
}
