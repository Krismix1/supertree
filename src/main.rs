use std::env;

use color_eyre::{eyre::Result, Report};

mod worktree;

#[derive(Debug)]
struct Config {
    branch_name: String,
}

impl Config {
    fn build(args: Vec<String>) -> Result<Self> {
        let branch_name = Config::get_positional_arg(&args, 0)?;
        Ok(Self { branch_name })
    }

    fn get_positional_arg(args: &[String], index: usize) -> Result<String> {
        let position = index + 1;
        let arg = args
            .get(index)
            .ok_or(Report::msg(format!(
                "expected argument at position {position}"
            )))?
            .to_owned();

        Ok(arg)
    }
}

fn helper(config: &Config) -> Result<()> {
    let repo = worktree::get_repo()?;
    worktree::create_worktree(&repo, &config.branch_name)?;

    Ok(())
}

fn main() -> Result<()> {
    color_eyre::install()?;
    let args: Vec<_> = env::args().skip(1).collect();
    let config = Config::build(args).unwrap();
    helper(&config)?;

    Ok(())
}
