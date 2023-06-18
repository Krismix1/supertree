use clap::Parser;
use cli::{Commands, SupertreeCli};
use color_eyre::eyre::Result;

mod cli;
mod tasks;
mod worktree;

fn main() -> Result<()> {
    color_eyre::install()?;

    let cli_config = SupertreeCli::parse();
    let projects_config = tasks::load_from_config_file()?;

    let directory = std::env::current_dir()?;
    let current_dir_name = directory.file_name().unwrap().to_str().unwrap();

    let project_config = projects_config
        .project_configs
        .get(current_dir_name)
        .unwrap_or(&projects_config.default_config);

    match cli_config.command {
        Commands::New(args) => {
            let repo = worktree::get_repo_curr_dir()?;
            return worktree::create_worktree(&repo, &args, project_config);
        }
        Commands::Config => {}
    }

    Ok(())
}
