use git2::Repository;
use std::fs;
use std::path::Path;
use std::{env, error::Error};

#[derive(Debug)]
struct Config {
    branch_name: String,
}

impl Config {
    fn build(args: Vec<String>) -> Result<Self, Box<dyn Error>> {
        let branch_name = Config::get_positional_arg(&args, 0)?;
        return Ok(Self { branch_name });
    }

    fn get_positional_arg(args: &Vec<String>, index: usize) -> Result<String, Box<dyn Error>> {
        let position = index + 1;
        let arg = args
            .get(index)
            .ok_or(format!("expected argument at position {position}"))?
            .to_owned();

        return Ok(arg);
    }
}

fn get_repo() -> Result<Repository, git2::Error> {
    let path = "/tmp/dummy_repo";
    let repo = Repository::init(path)?;
    let file_path = path.to_owned() + "/dummy.txt";
    fs::write(&file_path, "mock").unwrap();
    let mut index = repo.index()?;
    index.add_path(Path::new("dummy.txt"))?;
    // index.write()?;

    {
        let tree_oid = index.write_tree()?;
        let tree = repo.find_tree(tree_oid)?;

        let parent_commit = match repo.revparse_single("HEAD") {
            Ok(obj) => Some(obj.into_commit().unwrap()),
            // First commit so no parent commit
            Err(e) if e.code() == git2::ErrorCode::NotFound => None,
            Err(e) => return Err(e),
        };

        let mut parents = Vec::new();
        if parent_commit.is_some() {
            parents.push(parent_commit.as_ref().unwrap());
        }

        let signature = repo.signature()?;
        repo.commit(
            Some("HEAD"),
            &signature,
            &signature,
            "My message",
            &tree,
            &parents,
        )?;
        index.write()?;
    } // HACK: Is this needed due to a bad lifetime annotation?

    return Ok(repo);
}

fn helper(config: &Config) -> Result<(), git2::Error> {
    let repo = get_repo()?;
    let commit = repo.find_commit(repo.head()?.peel_to_commit()?.id())?;
    let _branch = repo.branch(&config.branch_name, &commit, true)?;

    Ok(())
}

fn main() {
    let args: Vec<_> = env::args().skip(1).collect();
    let config = Config::build(args).unwrap();
    helper(&config).unwrap();
}
