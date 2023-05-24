use git2::Repository;
use std::fs as std_fs;
use std::{error::Error, path::PathBuf};

use self::fs::CopyTargets;

mod fs;

pub fn get_repo() -> Result<Repository, git2::Error> {
    let path = "/tmp/dummy_repo";
    let repo = Repository::init(path)?;

    Ok(repo)
}

pub fn create_worktree(repo: &Repository, branch_name: &str) -> Result<(), Box<dyn Error>> {
    let target_dir = new_worktree(repo, branch_name)?;
    prepare_worktree(repo, target_dir)?;

    Ok(())
}

fn new_worktree(repo: &Repository, branch_name: &str) -> Result<PathBuf, Box<dyn Error>> {
    // TODO: Actually invoke git to create a worktree
    // For now it will just create the folder path based on the branch name
    // TODO: Probably good to validate the branch name does not try to traverse the fs

    let commit = repo.find_commit(repo.head()?.peel_to_commit()?.id())?;
    let _branch = repo.branch(branch_name, &commit, true)?;

    let repo_root = get_root_path(repo)?;
    let path = repo_root.join(branch_name);
    std_fs::create_dir_all(path.clone())?;

    Ok(path)
}

fn prepare_worktree(repo: &Repository, target_dir: PathBuf) -> Result<(), Box<dyn Error>> {
    let repo_root = get_root_path(repo)?;

    let source_dir = repo_root.join("master");
    let targets = [
        CopyTarget::new("**REMOVED**".to_string(), true),
        CopyTarget::new("**REMOVED**".to_string(), true),
    ];
    fs::copy_files(&source_dir, &target_dir, &targets)?;

    Ok(())
}

fn get_root_path(repo: &Repository) -> Result<PathBuf, Box<dyn Error>> {
    // let path = env::current_dir()?;
    let repo_root = repo.path().parent().map(|f| f.to_path_buf());

    repo_root.ok_or("Failed to get root path".into())
}
