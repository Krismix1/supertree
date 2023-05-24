use std::{env, error::Error};

mod worktree;

#[derive(Debug)]
struct Config {
    branch_name: String,
}

impl Config {
    fn build(args: Vec<String>) -> Result<Self, Box<dyn Error>> {
        let branch_name = Config::get_positional_arg(&args, 0)?;
        Ok(Self { branch_name })
    }

    fn get_positional_arg(args: &[String], index: usize) -> Result<String, Box<dyn Error>> {
        let position = index + 1;
        let arg = args
            .get(index)
            .ok_or(format!("expected argument at position {position}"))?
            .to_owned();

        Ok(arg)
    }
}

fn helper(config: &Config) -> Result<(), Box<dyn Error>> {
    let repo = worktree::get_repo()?;
    worktree::create_worktree(&repo, &config.branch_name)?;

    Ok(())
}

fn main() {
    let args: Vec<_> = env::args().skip(1).collect();
    let config = Config::build(args).unwrap();
    helper(&config).unwrap();
}
