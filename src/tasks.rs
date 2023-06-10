use directories::ProjectDirs;
use std::{
    collections::BTreeMap,
    env,
    fs::{self, File},
    io::{BufReader, BufWriter},
    path::{Path, PathBuf},
};

use std::io::ErrorKind::NotFound;

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
pub struct ProjectConfig {
    pub primary_branch: String,
}

impl Default for ProjectConfig {
    fn default() -> Self {
        Self {
            primary_branch: "master".to_owned(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(deny_unknown_fields)]
pub struct RootConfig {
    #[serde(default, rename = "default")]
    pub default_config: ProjectConfig,

    #[serde(rename = "projects")]
    pub project_configs: BTreeMap<String, ProjectConfig>,
}

fn store_config(config: &RootConfig, path: &Path) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();

    let fp = File::create(path).unwrap();
    let writer = BufWriter::new(fp);
    serde_yaml::to_writer(writer, config).unwrap();
}

pub fn load_from_config_file() -> RootConfig {
    let config_path: PathBuf = ProjectDirs::from("", "", env!("CARGO_PKG_NAME"))
        .unwrap()
        .config_dir()
        .join("config.yaml");

    let fp = match File::open(config_path.clone()) {
        Ok(fp) => Ok(fp),
        Err(ref e) if e.kind() == NotFound => {
            let config = Default::default();
            store_config(&config, &config_path);
            return config;
        }
        e => e,
    }
    .expect("failed to read config file");

    let reader = BufReader::new(fp);
    let config: Result<RootConfig, serde_yaml::Error> = serde_yaml::from_reader(reader);

    config.expect("failed to parse config file")
}
