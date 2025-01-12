use git2::{Repository, RepositoryInitOptions, Signature, WorktreeAddOptions};
use std::path::PathBuf;
use tempfile::TempDir;

pub struct TestRepo {
    _temp_dir: TempDir, // Keep TempDir alive to prevent cleanup
    pub bare_path: PathBuf,
    pub master_path: PathBuf,
}

impl TestRepo {
    pub fn new() -> color_eyre::Result<Self> {
        let temp_dir = tempfile::tempdir()?;
        let bare_path = temp_dir.path().join("repo.git");
        let master_path = temp_dir.path().join("master");

        // Create bare repo
        let repo = Repository::init_bare(&bare_path)?;
        // Repository::init_opts(&bare_path, &RepositoryInitOptions::new().bare(true).initial_head(head))?;

        // Create an empty tree
        let empty_tree = repo.treebuilder(None)?.write()?;

        // Create initial commit with empty tree
        let signature = Signature::now("Test User", "test@example.com")?;
        let commit = repo.commit(
            Some("refs/heads/master"),
            &signature,
            &signature,
            "Initial empty commit",
            &repo.find_tree(empty_tree)?,
            &[],
        )?;

        // Create master worktree with specific commit
        let mut opts = WorktreeAddOptions::new();
        let binding = repo.find_reference("refs/heads/master")?;
        opts.reference(Some(&binding));
        let master_tree = repo.worktree("master", &master_path, Some(&opts))?;
        let master_repo = Repository::open(master_tree.path())?;

        Ok(Self {
            _temp_dir: temp_dir,
            bare_path,
            master_path,
        })
    }

    pub fn create_test_file(&self, name: &str, content: &str) -> color_eyre::Result<()> {
        let path = self.master_path.join(name);
        std::fs::write(path, content)?;

        let repo = Repository::open(&self.master_path)?;
        let mut index = repo.index()?;
        index.add_path(std::path::Path::new(name))?;
        index.write()?;

        let tree_id = index.write_tree()?;
        let tree = repo.find_tree(tree_id)?;
        let signature = Signature::now("Test User", "test@example.com")?;
        let parent = repo.head()?.peel_to_commit()?;

        repo.commit(
            Some("HEAD"),
            &signature,
            &signature,
            "Add test file",
            &tree,
            &[&parent],
        )?;

        Ok(())
    }

    pub fn create_gitignore(&self, content: &str) -> color_eyre::Result<()> {
        self.create_test_file(".gitignore", content)
    }
}
