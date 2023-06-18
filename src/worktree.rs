use color_eyre::eyre::{Context, Result};
use color_eyre::Report;
use git2::{Branch, BranchType, Repository, WorktreeAddOptions};
use std::fs;
use std::path::PathBuf;

use crate::cli::CliConfig;
use crate::tasks::{files, shell};
use crate::tasks::{ProjectConfig, Task};

pub fn get_repo_curr_dir() -> Result<Repository> {
    let path = std::env::current_dir()?;
    let repo = Repository::open(path)?;

    Ok(repo)
}

pub fn create_worktree(
    repo: &Repository,
    cli_config: &CliConfig,
    project_config: &ProjectConfig,
) -> Result<()> {
    let branch_name = &cli_config.branch_name;
    let remote_branch: Option<&str> = cli_config.remote_branch();

    let (branch_type, ref_branch) = remote_branch
        .map(|rb| if rb.is_empty() { branch_name } else { rb })
        .map_or_else(
            || (BranchType::Local, project_config.primary_branch.clone()),
            |br| {
                (
                    BranchType::Remote,
                    format!("{}/{br}", project_config.primary_remote),
                )
            },
        );

    let ref_branch = repo.find_branch(&ref_branch, branch_type).context(format!(
        "{:?} ref branch {} not found",
        branch_type, branch_name
    ))?;

    println!(
        "Using {:?} ref branch {} for checkout",
        branch_type,
        ref_branch.name().unwrap().unwrap()
    );

    let target_dir = new_worktree(
        repo,
        branch_name,
        &ref_branch,
        &project_config.primary_remote,
    )?;

    if !cli_config.skip_tasks {
        prepare_worktree(repo, target_dir, project_config)?;
    }

    Ok(())
}

fn new_worktree(
    repo: &Repository,
    branch_name: &str,
    ref_branch: &Branch,
    remote: &str,
) -> Result<PathBuf> {
    let mut new_branch = repo
        .branch(branch_name, &ref_branch.get().peel_to_commit()?, false)
        .wrap_err("Failed to create new branch")?;

    new_branch
        .set_upstream(Some(remote))
        .context("Failed to set upstream for branch")?;

    let mut worktree_add_options = WorktreeAddOptions::new();
    worktree_add_options.reference(Some(new_branch.get()));

    let repo_root = get_root_path(repo)?;
    let worktree_path = repo_root.join(branch_name); // TODO: Perhaps split by '/' and then join parts to path

    let parent_dir = worktree_path
        .parent()
        .expect("expected to extract parent dir");

    fs::create_dir_all(parent_dir).context(format!(
        "Failed to create directory {}",
        parent_dir.display()
    ))?;

    // worktree name is used to create directory .git/worktrees/<name>
    let worktree_name = branch_name.replace(std::path::MAIN_SEPARATOR, "_");

    let worktree = repo.worktree(
        &worktree_name,
        worktree_path.as_path(),
        Some(&worktree_add_options),
    )?;

    assert_eq!(worktree_path, worktree.path().to_path_buf());

    Ok(worktree_path)
}

fn prepare_worktree(repo: &Repository, target_dir: PathBuf, config: &ProjectConfig) -> Result<()> {
    let repo_root = get_root_path(repo)?;

    let source_dir = repo_root.join(&config.primary_branch);
    let source_dir = if source_dir.exists() {
        source_dir
    } else {
        println!(
            "{} not found, using repo root {}",
            source_dir.display(),
            repo_root.display()
        );
        repo_root
    };

    let target_dir = target_dir.canonicalize()?;
    for task in &config.tasks {
        match task {
            Task::Shell(config) => shell::run_shell(config, &target_dir)?,
            Task::CopyPath(config) => files::copy_path(config, &source_dir, &target_dir)?,
        }
    }

    Ok(())
}

fn get_root_path(repo: &Repository) -> Result<PathBuf> {
    let repo_root = repo.path().parent().map(|f| f.to_path_buf());

    repo_root.ok_or(Report::msg("Failed to get root path"))
}
