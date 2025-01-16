use color_eyre::eyre::{Context, Result};
use color_eyre::{Report, Section};
use git2::{Branch, BranchType, Repository, WorktreeAddOptions};
use std::fs;
use std::path::PathBuf;

use crate::cli::NewWorktreeArgs;
use crate::tasks::{files, shell};
use crate::tasks::{ProjectConfig, Task};

pub fn get_repo_curr_dir() -> Result<Repository> {
    let repo = Repository::open_from_env()?;

    Ok(repo)
}

pub fn create_worktree(
    repo: &Repository,
    new_worktree_args: &NewWorktreeArgs,
    project_config: &ProjectConfig,
) -> Result<()> {
    let branch_name = &new_worktree_args.branch_name;
    let remote_branch: Option<&str> = new_worktree_args.remote_branch();

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
        "{:?} ref branch '{}' not found",
        branch_type, branch_name
    ))?;

    eprintln!(
        "Using {:?} ref branch {} for checkout",
        branch_type,
        ref_branch.name().unwrap().unwrap()
    );

    let target_dir = new_worktree(
        repo,
        branch_name,
        &ref_branch,
        remote_branch.map(|_| project_config.primary_remote.as_str()),
    )?;

    if !new_worktree_args.skip_tasks {
        prepare_worktree(repo, target_dir, project_config)?;
    }

    Ok(())
}

fn new_worktree(
    repo: &Repository,
    branch_name: &str,
    ref_branch: &Branch,
    remote: Option<&str>,
) -> Result<PathBuf> {
    let repo_root = get_root_path(repo)?;
    // convert "/" to OS specific path separator
    let worktree_path = repo_root.join(branch_name.split('/').collect::<PathBuf>());
    eprintln!("Creating worktree at {}", worktree_path.display());

    // must create parent directory for nested folders
    let parent_dir = worktree_path
        .parent()
        .ok_or_else(|| Report::msg("expected to extract parent dir"))?;
    fs::create_dir_all(parent_dir).context(format!(
        "Failed to create directory {}",
        parent_dir.display()
    ))?;

    let target_branch = match repo.find_branch(branch_name, BranchType::Local) {
        Ok(b) => b,
        Err(err) => {
            eprintln!("Failed to find branch: {:?}", err);
            let mut target_branch = repo
                .branch(branch_name, &ref_branch.get().peel_to_commit()?, false)
                .wrap_err("Failed to create new branch")?;

            if remote.is_some() {
                target_branch
                    // passing None unsets the remote...
                    // but I want to keep it for existing branches
                    .set_upstream(
                        Some(ref_branch.name().unwrap().unwrap()),
                    )
                    .with_context(|| format!("Failed to set upstream for branch '{}' to '{}'", branch_name, ref_branch.name().unwrap().unwrap()))?;
            }
            target_branch
        }
    };

    let mut worktree_add_options = WorktreeAddOptions::new();
    worktree_add_options.reference(Some(target_branch.get()));

    // worktree name is used to create directory .git/worktrees/<name>
    let worktree_name = branch_name.replace(std::path::MAIN_SEPARATOR, "_");

    let worktree = repo
        .worktree(
            &worktree_name,
            worktree_path.as_path(),
            Some(&worktree_add_options),
        )
        .with_context(|| "failed to create worktree")?;

    assert_eq!(worktree_path, worktree.path().to_path_buf());

    Ok(worktree_path)
}

fn prepare_worktree(repo: &Repository, target_dir: PathBuf, config: &ProjectConfig) -> Result<()> {
    let repo_root = get_root_path(repo)?;

    let source_dir = repo_root.join(&config.primary_branch);
    let source_dir = if source_dir.exists() {
        source_dir
    } else {
        eprintln!(
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

pub fn get_root_path(repo: &Repository) -> Result<PathBuf> {
    // https://github.com/rust-lang/git2-rs/pull/1079/files
    // Perhaps the new function can simplify this part
    if repo.is_worktree() {
        let workdir = repo.workdir().unwrap(); // /tmp/repo/feature/nested/
        let worktree_path = repo.path(); // /tmp/repo/.bare/worktrees/nested/
        let mut root_path = PathBuf::new();
        for component in worktree_path.iter() {
            root_path.push(component);
            if !workdir.starts_with(&root_path) {
                break;
            }
        }
        let repo_root = root_path.parent().map(|f| f.to_path_buf());
        repo_root
            .ok_or_else(|| Report::msg("Failed to get root path from worktree workdir"))
            .with_note(|| {
                format!(
                    "workdir: {}, worktree: {}, root path: {}",
                    workdir.display(),
                    worktree_path.display(),
                    root_path.display()
                )
            })
    } else if repo.is_bare() {
        // repo.path(): /tmp/repo/.bare/
        let repo_root = repo.path().parent().map(|f| f.to_path_buf());
        repo_root
            .ok_or_else(|| Report::msg("Failed to get root path from bare repository"))
            .with_note(|| format!("repo path: {}", repo.path().display()))
    } else {
        Err(Report::msg("Not a worktree or bare repo").with_note(|| {
            format!(
                "Command can be only from a bare repository or a work tree. Work directory has been identified to: {}",
                repo.workdir()
                    .map(|p| p.display().to_string())
                    .unwrap_or("None".to_owned())
            )
        }))
    }
}
