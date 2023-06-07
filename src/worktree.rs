use self::{commands::Commands, fs::CopyTargets};
use color_eyre::eyre::{Context, Result};
use color_eyre::Report;
use git2::{BranchType, Reference, Repository, WorktreeAddOptions};
use std::path::PathBuf;
use std::process::Command;

mod fs;

mod commands {
    pub enum Commands {
        Shell(String),
    }
}

const MASTER_BRANCH: &str = "master";

pub fn get_repo() -> Result<Repository> {
    let path = std::env::current_dir()?;
    let repo = Repository::init(path)?;

    Ok(repo)
}

pub fn create_worktree(repo: &Repository, branch_name: &str) -> Result<()> {
    let target_dir = new_worktree(repo, branch_name)?;
    prepare_worktree(repo, target_dir)?;

    Ok(())
}

fn new_worktree(repo: &Repository, branch_name: &str) -> Result<PathBuf> {
    if !Reference::is_valid_name(branch_name) {
        return Result::Err(Report::msg(format!(
            "Branch name '{branch_name}' is not valid"
        )));
    }

    let mut worktree_add_options = WorktreeAddOptions::new();

    // TODO: Maybe support picking a different source ref
    let ref_branch = repo.find_branch(MASTER_BRANCH, BranchType::Local)?;
    let new_branch = repo
        .branch(branch_name, &ref_branch.get().peel_to_commit()?, false)
        .wrap_err("Failed to create new branch")?;
    worktree_add_options.reference(Some(new_branch.get()));

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

fn prepare_worktree(repo: &Repository, target_dir: PathBuf) -> Result<()> {
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

    let commands = [Commands::Shell("npm run prepare".to_string())];

    let target_dir = target_dir.canonicalize()?;
    for command in commands {
        match command {
            Commands::Shell(cmd) => {
                println!("Running command `{cmd}`");
                Command::new("bash")
                    .arg("-c")
                    .arg(cmd)
                    .current_dir(&target_dir)
                    .output()?;
            }
        }
    }

    Ok(())
}

fn get_root_path(repo: &Repository) -> Result<PathBuf> {
    let repo_root = repo.path().parent().map(|f| f.to_path_buf());

    repo_root.ok_or(Report::msg("Failed to get root path"))
}
