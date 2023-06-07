use clap::Parser;
use cli::CliConfig;
use color_eyre::eyre::Result;

mod cli;
mod worktree;

fn helper(config: &CliConfig) -> Result<()> {
    let repo = worktree::get_repo()?;
    worktree::create_worktree(&repo, &config.branch_name)?;

    Ok(())
}

fn main() -> Result<()> {
    color_eyre::install()?;
    let config = CliConfig::parse();

    helper(&config)?;

    Ok(())
}
