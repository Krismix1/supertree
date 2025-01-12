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
    #[serde(default)]
    pub missing_okay: bool,
}

fn symlink(src: &Path, tgt: &Path) -> color_eyre::Result<()> {
    eprintln!("Symlinking {} to {}", src.display(), tgt.display());

    // TODO: Support softlink on Windows?
    // https://doc.rust-lang.org/std/os/windows/fs/fn.symlink_file.html
    unix_fs::symlink(src, tgt).context(format!(
        "failed to symlink {} to {}",
        src.display(),
        tgt.display(),
    ))?;

    Ok(())
}

fn copy_file(src: &Path, tgt: &Path, verbose: bool) -> color_eyre::Result<()> {
    if verbose {
        eprintln!("Copying {} to {}", src.display(), tgt.display());
    }

    fs::copy(src, tgt).context(format!(
        "failed to copy {} to {}",
        src.display(),
        tgt.display(),
    ))?;

    Ok(())
}

fn copy_dir(src: &Path, tgt: &Path, verbose: bool) -> color_eyre::Result<()> {
    if verbose {
        eprintln!("Copying {} to {}", src.display(), tgt.display());
    }

    fs::create_dir_all(tgt)?;

    let file_iter = src
        .read_dir()
        .context(format!(
            "Failed to list files in directory {}",
            src.display()
        ))?
        .flatten();

    for entry in file_iter {
        let entry = entry.path();
        let target_entry = tgt.join(entry.strip_prefix(src)?);
        if !entry.is_dir() {
            copy_file(&entry, &target_entry, false)?;
        } else {
            copy_dir(&entry, &target_entry, false)?;
        }
    }

    Ok(())
}

pub fn copy_path(
    copy_config: &CopyPathConfig,
    source_dir: &Path,
    target_dir: &Path,
) -> color_eyre::Result<()> {
    let source_entry = source_dir.join(&copy_config.source);
    if copy_config.missing_okay && !source_entry.exists() {
        eprintln!("Skipping {} as it is missing", source_entry.display());
        return Ok(());
    }

    let target_entry = target_dir.join(&copy_config.source);

    if copy_config.symlink {
        symlink(&source_entry, &target_entry)?;
    } else if source_entry.is_dir() {
        copy_dir(&source_entry, &target_entry, true)?;
    } else {
        copy_file(&source_entry, &target_entry, true)?;
    }

    Ok(())
}
