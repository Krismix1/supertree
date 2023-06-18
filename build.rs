use clap::CommandFactory;
use clap::ValueEnum;
use clap_complete::{generate_to, Shell};
use std::env;

include!("src/cli.rs");

fn main() -> Result<(), Error> {
    let outdir = env::var_os("OUT_DIR").expect("expected to get output dir path");

    let mut cmd = SupertreeCli::command();
    let cmd_name = cmd.get_name().to_string();
    for &shell in Shell::value_variants() {
        generate_to(shell, &mut cmd, &cmd_name, outdir.clone())?;
    }

    Ok(())
}
