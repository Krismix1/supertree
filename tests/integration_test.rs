use assert_fs::prelude::*;

use common::TestRepo;
use git2::Repository;
use supertree::{
    cli::{CliArgs, NewWorktreeArgs},
    tasks::ProjectConfig,
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
    std::env::set_current_dir(&test_repo.master_path)?;

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
