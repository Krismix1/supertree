use clap::{
    command, error::Error, Arg, ArgMatches, Args, Command, FromArgMatches, Parser, Subcommand,
};

#[derive(Debug, PartialEq, Eq)]
pub struct CliArgs {
    pub remote_branch: Option<String>,
}

const REMOTE_BRANCH_NAME: &str = "REMOTE_BRANCH";

impl FromArgMatches for CliArgs {
    fn from_arg_matches(matches: &ArgMatches) -> Result<Self, Error> {
        let mut matches = matches.clone();
        Self::from_arg_matches_mut(&mut matches)
    }

    fn from_arg_matches_mut(matches: &mut ArgMatches) -> Result<Self, Error> {
        Ok(Self {
            remote_branch: matches.remove_one::<String>(REMOTE_BRANCH_NAME),
        })
    }

    fn update_from_arg_matches(&mut self, matches: &ArgMatches) -> Result<(), Error> {
        let mut matches = matches.clone();
        self.update_from_arg_matches_mut(&mut matches)
    }

    fn update_from_arg_matches_mut(&mut self, matches: &mut ArgMatches) -> Result<(), Error> {
        if let Some(remote_branch) = matches.remove_one::<String>(REMOTE_BRANCH_NAME) {
            self.remote_branch = Some(remote_branch);
        }
        Ok(())
    }
}

impl Args for CliArgs {
    fn augment_args(cmd: Command) -> Command {
        Self::augment_args_for_update(cmd)
    }

    fn augment_args_for_update(cmd: Command) -> Command {
        cmd.arg(
            Arg::new(REMOTE_BRANCH_NAME)
                .short('r')
                .long("remote")
                .help("The remote branch to check out, if any. If supplied without a value (i.e. as a flag), then it will default to the BRANCH_NAME argument")
                .num_args(0..=1)
                .require_equals(true)
                // https://docs.rs/clap/latest/clap/struct.Arg.html#method.default_missing_value
                // allows using the argument both as a flag and as an argument
                .default_missing_value(""),
        )
    }
}

#[derive(Parser, Debug, PartialEq, Eq)]
#[command(author, version, about, long_about = None)]
pub struct NewWorktreeArgs {
    /// the name of the branch to create
    pub branch_name: String,

    /// skip running tasks
    #[arg(short, long, default_value_t = false)]
    pub skip_tasks: bool,

    // https://docs.rs/clap/latest/clap/_derive/index.html#flattening-hand-implemented-args-into-a-derived-application
    // default_missing_value is only supported via the Arg struct
    #[command(flatten)]
    pub more_args: CliArgs,
}

impl NewWorktreeArgs {
    pub fn remote_branch(&self) -> Option<&str> {
        self.more_args.remote_branch.as_deref()
    }
}

/// A CLI to improve the DX around git worktree
#[derive(Debug, Parser)]
#[command(name = "supertree")]
#[command(author, version, about, long_about = None)]
pub struct SupertreeCli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand, PartialEq, Eq)]
pub enum Commands {
    /// Create new work tree. Creates default config if missing
    #[command(arg_required_else_help = true)]
    New(NewWorktreeArgs),

    /// Only generate default config (if missing) and exit
    Config,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_args() -> Result<(), Box<dyn std::error::Error>> {
        let args = SupertreeCli::try_parse_from(["supertree", "new", "feature"])?;
        assert_eq!(
            args.command,
            Commands::New(NewWorktreeArgs {
                branch_name: "feature".to_string(),
                skip_tasks: false,
                more_args: CliArgs {
                    remote_branch: None,
                },
            })
        );
        Ok(())
    }

    #[test]
    fn test_cli_args_with_remote() -> Result<(), Box<dyn std::error::Error>> {
        let args = SupertreeCli::try_parse_from(["supertree", "new", "-r=the_remote", "feature"])?;
        assert_eq!(
            args.command,
            Commands::New(NewWorktreeArgs {
                branch_name: "feature".to_string(),
                skip_tasks: false,
                more_args: CliArgs {
                    remote_branch: Some("the_remote".to_string()),
                },
            })
        );
        Ok(())
    }

    #[test]
    fn test_cli_args_with_remote_as_flag() -> Result<(), Box<dyn std::error::Error>> {
        let args = SupertreeCli::try_parse_from(["supertree", "new", "-r", "feature"])?;
        assert_eq!(
            args.command,
            Commands::New(NewWorktreeArgs {
                branch_name: "feature".to_string(),
                skip_tasks: false,
                more_args: CliArgs {
                    remote_branch: Some("".to_string()),
                },
            })
        );
        Ok(())
    }
}
