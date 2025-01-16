use clap::Parser;
use color_eyre::eyre::Result;
use supertree::{
    cli::{Commands, SupertreeCli},
    tasks,
    worktree::{self, get_root_path},
};

fn main() -> Result<()> {
    color_eyre::install()?;

    let cli_config = SupertreeCli::parse();
    let projects_config = tasks::load_from_config_file()?;

    let repo = worktree::get_repo_curr_dir()?;
    let root_path = get_root_path(&repo)?;
    let current_dir_name = root_path.file_name().unwrap().to_str().unwrap();

    eprintln!("Detected project name to {}", current_dir_name);
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
