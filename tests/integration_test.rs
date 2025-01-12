use std::path::PathBuf;

use assert_fs::prelude::*;

use common::TestRepo;
use git2::Repository;
use supertree::{
    cli::{CliArgs, NewWorktreeArgs},
    tasks::{files::CopyPathConfig, ProjectConfig, Task},
    worktree,
};

mod common;

#[test]
fn test_create_worktree() -> color_eyre::Result<()> {
    color_eyre::install()?;

    // Setup test repository
    let test_repo = TestRepo::new()?;
    test_repo.create_test_file("test.txt", "Hello, World!")?;

    // Change to master worktree directory
    // std::env::set_current_dir(&test_repo.master_path)?;

    // Create new worktree using supertree
    let repo = Repository::open(&test_repo.master_path)?;
    let args = NewWorktreeArgs {
        branch_name: "feature/new-feature".to_string(),
        skip_tasks: true,
        more_args: CliArgs {
            remote_branch: None,
        },
    };
    let config = ProjectConfig::default();

    worktree::create_worktree(&repo, &args, &config)?;

    // Verify the new worktree
    let new_worktree_path = test_repo
        .bare_path
        .parent()
        .unwrap()
        .join("feature/new-feature");
    assert!(new_worktree_path.exists());
    assert!(new_worktree_path.join("test.txt").exists());
    assert_eq!(
        std::fs::read_to_string(new_worktree_path.join("test.txt"))?,
        "Hello, World!"
    );

    Ok(())
}

#[test]
fn test_copy_ignored_file() -> color_eyre::Result<()> {
    let test_repo = TestRepo::new()?;

    // Setup .gitignore
    test_repo.create_gitignore("ignore_me.txt\n")?;

    // Create the ignored file in master
    let ignored_path = test_repo.master_path.join("ignore_me.txt");
    std::fs::write(&ignored_path, "secret content")?;

    let repo = Repository::open(&test_repo.master_path)?;
    let args = NewWorktreeArgs {
        branch_name: "feature".to_string(),
        skip_tasks: false,
        more_args: CliArgs {
            remote_branch: None,
        },
    };
    let config = ProjectConfig {
        tasks: vec![Task::CopyPath(CopyPathConfig {
            source: PathBuf::from("ignore_me.txt"),
            symlink: false,
            missing_okay: false,
        })],
        ..Default::default()
    };

    // Create a new worktree with a task to copy the ignored file
    worktree::create_worktree(&repo, &args, &config)?;

    let new_worktree_path = test_repo.bare_path.parent().unwrap().join("feature");

    // Verify the file was copied
    let copied_content = std::fs::read_to_string(new_worktree_path.join("ignore_me.txt"))?;
    assert_eq!(copied_content, "secret content");

    Ok(())
}
