use std::path::PathBuf;

use assert_fs::prelude::*;

use common::TestRepo;
use git2::{build::CheckoutBuilder, BranchType, Repository, Signature};
use supertree::{
    cli::{CliArgs, NewWorktreeArgs},
    tasks::{files::CopyPathConfig, ProjectConfig, Task},
    worktree,
};

mod common;

#[test]
fn test_create_worktree() -> Result<(), Box<dyn std::error::Error>> {
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
fn test_copy_ignored_file() -> Result<(), Box<dyn std::error::Error>> {
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

#[test]
fn test_create_worktree_from_remote_branch() -> Result<(), Box<dyn std::error::Error>> {
    // Create the "remote" repository
    let remote_repo = TestRepo::new()?;
    remote_repo.create_test_file("master.txt", "master content")?;

    // Create and setup feature branch in remote
    let repo = Repository::open(&remote_repo.master_path)?;
    let head = repo.head()?.peel_to_commit()?;
    repo.branch("feature", &head, false)?;
    let feature_tree = repo.worktree(
        "feature",
        &remote_repo.bare_path.parent().unwrap().join("feature"),
        None,
    )?;
    let feature_repo = Repository::open(feature_tree.path())?;

    // Add file to feature branch
    std::fs::write(feature_tree.path().join("feature.txt"), "feature content")?;
    let mut index = feature_repo.index()?;
    index.add_path(std::path::Path::new("feature.txt"))?;
    index.write()?;
    let tree_id = index.write_tree()?;
    let tree = feature_repo.find_tree(tree_id)?;
    let parent = feature_repo.head()?.peel_to_commit()?;
    let signature = Signature::now("Test User", "test@example.com")?;
    feature_repo.commit(
        Some("HEAD"),
        &signature,
        &signature,
        "Add feature file",
        &tree,
        &[&parent],
    )?;

    // Create second repo by cloning first repo
    let local_repo = TestRepo::clone_from(&remote_repo.bare_path)?;

    // Create new worktree from remote feature branch
    let repo = Repository::open(&local_repo.master_path)?;
    repo.find_branch("feature", BranchType::Local)?.delete()?;

    let args = NewWorktreeArgs {
        branch_name: "feature".to_string(),
        skip_tasks: true,
        more_args: CliArgs {
            remote_branch: Some("".to_string()),
        },
    };
    let config = ProjectConfig::default();

    worktree::create_worktree(&repo, &args, &config)?;

    // Verify the worktree was created with content from feature branch
    let new_worktree_path = local_repo.bare_path.parent().unwrap().join("feature");
    assert!(new_worktree_path.exists());
    assert!(new_worktree_path.join("feature.txt").exists());
    assert_eq!(
        std::fs::read_to_string(new_worktree_path.join("feature.txt"))?,
        "feature content"
    );

    let branch = repo.find_branch("feature", BranchType::Local);
    assert!(branch.is_ok());
    assert!(branch.unwrap().upstream().is_ok());

    Ok(())
}

#[test]
fn test_create_worktree_from_remote_branch_with_conflicting_local(
) -> Result<(), Box<dyn std::error::Error>> {
    // Create the "remote" repository
    let remote_repo = TestRepo::new()?;
    remote_repo.create_test_file("master.txt", "master content")?;

    // Create and setup feature branch in remote
    let repo = Repository::open(&remote_repo.master_path)?;
    let head = repo.head()?.peel_to_commit()?;
    repo.branch("feature", &head, false)?;
    let feature_tree = repo.worktree(
        "feature",
        &remote_repo.bare_path.parent().unwrap().join("feature"),
        None,
    )?;
    let feature_repo = Repository::open(feature_tree.path())?;

    // Add file to feature branch
    std::fs::write(feature_tree.path().join("feature.txt"), "feature content")?;
    let mut index = feature_repo.index()?;
    index.add_path(std::path::Path::new("feature.txt"))?;
    index.write()?;
    let tree_id = index.write_tree()?;
    let tree = feature_repo.find_tree(tree_id)?;
    let parent = feature_repo.head()?.peel_to_commit()?;
    let signature = Signature::now("Test User", "test@example.com")?;
    feature_repo.commit(
        Some("HEAD"),
        &signature,
        &signature,
        "Add feature file",
        &tree,
        &[&parent],
    )?;

    // Create second repo by cloning first repo
    let local_repo = TestRepo::clone_from(&remote_repo.bare_path)?;

    // Create new worktree from remote feature branch
    let repo = Repository::open(&local_repo.master_path)?;

    let args = NewWorktreeArgs {
        branch_name: "feature".to_string(),
        skip_tasks: true,
        more_args: CliArgs {
            remote_branch: Some("feature".to_string()),
        },
    };
    let config = ProjectConfig::default();

    worktree::create_worktree(&repo, &args, &config)?;

    // Verify the worktree was created with content from feature branch
    let new_worktree_path = local_repo.bare_path.parent().unwrap().join("feature");
    assert!(new_worktree_path.exists());
    assert!(new_worktree_path.join("feature.txt").exists());
    assert_eq!(
        std::fs::read_to_string(new_worktree_path.join("feature.txt"))?,
        "feature content"
    );

    Ok(())
}

#[test]
fn test_create_worktree_with_existing_local() -> Result<(), Box<dyn std::error::Error>> {
    let test_repo = TestRepo::new()?;
    test_repo.create_test_file("master.txt", "master content")?;

    // Create and setup feature branch in remote
    let repo = Repository::open(&test_repo.master_path)?;
    let head = repo.head()?.peel_to_commit()?;
    repo.branch("feature", &head, false)?;

    checkout_branch(&repo, "feature")?;

    // Add file to feature branch
    std::fs::write(
        repo.workdir().unwrap().join("feature.txt"),
        "feature content",
    )?;
    let mut index = repo.index()?;
    index.add_path(std::path::Path::new("feature.txt"))?;
    index.write()?;
    let tree_id = index.write_tree()?;
    let tree = repo.find_tree(tree_id)?;
    let parent = repo.head()?.peel_to_commit()?;
    let signature = Signature::now("Test User", "test@example.com")?;
    repo.commit(
        Some("HEAD"),
        &signature,
        &signature,
        "Add feature file",
        &tree,
        &[&parent],
    )?;

    // Checkout back to master
    checkout_branch(&repo, "master")?;

    let args = NewWorktreeArgs {
        branch_name: "feature".to_string(),
        skip_tasks: true,
        more_args: CliArgs {
            remote_branch: None,
        },
    };
    let config = ProjectConfig::default();

    worktree::create_worktree(&repo, &args, &config)?;

    // Verify the worktree was created with content from feature branch
    let new_worktree_path = test_repo.bare_path.parent().unwrap().join("feature");
    assert!(new_worktree_path.exists());
    assert!(new_worktree_path.join("feature.txt").exists());
    assert_eq!(
        std::fs::read_to_string(new_worktree_path.join("feature.txt"))?,
        "feature content"
    );

    Ok(())
}

#[test]
fn test_create_worktree_from_remote_branch_with_new_name() -> Result<(), Box<dyn std::error::Error>>
{
    // Create the "remote" repository
    let remote_repo = TestRepo::new()?;
    remote_repo.create_test_file("master.txt", "master content")?;

    // Create and setup feature branch in remote
    let repo = Repository::open(&remote_repo.master_path)?;
    let head = repo.head()?.peel_to_commit()?;
    repo.branch("feature", &head, false)?;
    let feature_tree = repo.worktree(
        "feature",
        &remote_repo.bare_path.parent().unwrap().join("feature"),
        None,
    )?;
    let feature_repo = Repository::open(feature_tree.path())?;

    // Add file to feature branch
    std::fs::write(feature_tree.path().join("feature.txt"), "feature content")?;
    let mut index = feature_repo.index()?;
    index.add_path(std::path::Path::new("feature.txt"))?;
    index.write()?;
    let tree_id = index.write_tree()?;
    let tree = feature_repo.find_tree(tree_id)?;
    let parent = feature_repo.head()?.peel_to_commit()?;
    let signature = Signature::now("Test User", "test@example.com")?;
    feature_repo.commit(
        Some("HEAD"),
        &signature,
        &signature,
        "Add feature file",
        &tree,
        &[&parent],
    )?;

    // Create second repo by cloning first repo
    let local_repo = TestRepo::clone_from(&remote_repo.bare_path)?;

    // Create new worktree from remote feature branch
    let repo = Repository::open(&local_repo.master_path)?;
    repo.find_branch("feature", BranchType::Local)?.delete()?;

    let args = NewWorktreeArgs {
        branch_name: "new-feature".to_string(),
        skip_tasks: true,
        more_args: CliArgs {
            remote_branch: Some("feature".to_string()),
        },
    };
    let config = ProjectConfig::default();

    worktree::create_worktree(&repo, &args, &config)?;

    // Verify the worktree was created with content from feature branch
    let new_worktree_path = local_repo.bare_path.parent().unwrap().join("new-feature");
    assert!(new_worktree_path.exists());
    assert!(new_worktree_path.join("feature.txt").exists());
    assert_eq!(
        std::fs::read_to_string(new_worktree_path.join("feature.txt"))?,
        "feature content"
    );

    let branch = repo.find_branch("new-feature", BranchType::Local);
    assert!(branch.is_ok());
    assert!(branch.unwrap().upstream().is_ok());

    Ok(())
}

fn checkout_branch(repo: &Repository, branch_name: &str) -> Result<(), git2::Error> {
    // Get the branch
    let branch = repo.find_branch(branch_name, BranchType::Local)?;
    let branch_ref = branch.get();

    // Set HEAD to point to the branch
    repo.set_head(branch_ref.name().unwrap())?;

    // Checkout the tree
    repo.checkout_head(Some(
        CheckoutBuilder::new()
            .force() // To make it like git checkout -f
            .remove_untracked(true), // Clean untracked files
    ))?;

    Ok(())
}
