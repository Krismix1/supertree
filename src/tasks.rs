use color_eyre::eyre::{Context, ContextCompat};
use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    env,
    fs::{self, File},
    io::{BufReader, BufWriter, ErrorKind::NotFound},
    path::{Path, PathBuf},
};

use self::{files::CopyPathConfig, shell::ShellConfig};

pub mod files;
pub mod shell;

#[derive(Serialize, Deserialize, Debug)]
// https://serde.rs/enum-representations.html#internally-tagged
#[serde(tag = "type")]
pub enum Task {
    CopyPath(CopyPathConfig),
    Shell(ShellConfig),
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct ProjectConfig {
    #[serde(default = "default_branch")]
    pub primary_branch: String,
    #[serde(default)]
    pub tasks: Vec<Task>,
}

fn default_branch() -> String {
    "master".to_string()
}

impl Default for ProjectConfig {
    fn default() -> Self {
        Self {
            primary_branch: default_branch(),
            tasks: Default::default(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct RootConfig {
    #[serde(default, rename = "default")]
    pub default_config: ProjectConfig,

    #[serde(rename = "projects")]
    pub project_configs: BTreeMap<String, ProjectConfig>,

    version: String,
}

impl Default for RootConfig {
    fn default() -> Self {
        Self {
            default_config: Default::default(),
            project_configs: Default::default(),
            version: "1".to_string(),
        }
    }
}

fn store_config(config: &RootConfig, path: &Path) -> color_eyre::Result<()> {
    fs::create_dir_all(
        path.parent()
            .context("Failed to extract path to config directory")?,
    )?;

    let fp = File::create(path).context("Failed to open config file")?;
    let writer = BufWriter::new(fp);
    serde_yaml::to_writer(writer, config).context("Failed to save config file")?;

    Ok(())
}

pub fn load_from_config_file() -> color_eyre::Result<RootConfig> {
    let config_path: PathBuf = ProjectDirs::from("", "", env!("CARGO_PKG_NAME"))
        .context("Failed to identify path to user config directory")?
        .config_dir()
        .join("config.yaml");

    let fp = match File::open(config_path.clone()) {
        Err(ref e) if e.kind() == NotFound => {
            let config = Default::default();
            store_config(&config, &config_path).context("Failed to save default config")?;

            return Ok(config);
        }
        v => v,
    }
    .context("Failed to read config file")?;

    let reader = BufReader::new(fp);
    let config: RootConfig =
        serde_yaml::from_reader(reader).context("Failed to parse config file")?;

    Ok(config)
}
