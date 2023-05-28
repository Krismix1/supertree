use self::fs::CopyTargets;
use git2::{BranchType, Reference, Repository, WorktreeAddOptions};
use std::{error::Error, path::PathBuf};

mod fs;

const MASTER_BRANCH: &str = "master";

pub fn get_repo() -> Result<Repository, Box<dyn Error>> {
    let path = std::env::current_dir()?;
    let repo = Repository::init(path)?;

    Ok(repo)
}

pub fn create_worktree(repo: &Repository, branch_name: &str) -> Result<(), Box<dyn Error>> {
    let target_dir = new_worktree(repo, branch_name)?;
    prepare_worktree(repo, target_dir)?;

    Ok(())
}

fn new_worktree(repo: &Repository, branch_name: &str) -> Result<PathBuf, Box<dyn Error>> {
    if !Reference::is_valid_name(branch_name) {
        return Err(format!("Branch name '{branch_name}' is not valid").into());
    }

    let mut worktree_add_options = WorktreeAddOptions::new();

    // TODO: Maybe support picking a different source ref
    let ref_branch = repo.find_branch(MASTER_BRANCH, BranchType::Local)?;
    let new_branch = repo.branch(branch_name, &ref_branch.get().peel_to_commit()?, false)?;
    worktree_add_options.reference(Some(new_branch.get()));

    // let mut path = env::current_dir()?;
    let repo_root = get_root_path(repo)?;
    let worktree_path = repo_root.join(branch_name); // TODO: Perhaps split by '/' and then join parts to path

    // worktree name is used to create folder .git/worktrees/<name>
    let worktree_name = branch_name.replace(std::path::MAIN_SEPARATOR, "_");

    let worktree = repo.worktree(
        &worktree_name,
        worktree_path.as_path(),
        Some(&worktree_add_options),
    )?;

    assert_eq!(worktree_path, worktree.path().to_path_buf());

    Ok(worktree_path)
}

fn prepare_worktree(repo: &Repository, target_dir: PathBuf) -> Result<(), Box<dyn Error>> {
    let repo_root = get_root_path(repo)?;

    let source_dir = repo_root.join(MASTER_BRANCH);
    let source_dir = if source_dir.exists() {
        source_dir
    } else {
        repo_root
    };

    let targets = [
        CopyTarget::new("**REMOVED**".to_string(), true),
        CopyTarget::new("**REMOVED**".to_string(), true),
        CopyTarget::new("**REMOVED**".to_string(), true),
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
