use std::collections::HashMap;
use std::fs::{create_dir_all, File};
use std::io::prelude::*;
use std::path::PathBuf;

use anyhow::{anyhow, bail, Result};
use config::Config;
use log::info;
use rand::Rng;
use serde_derive::{Deserialize, Serialize};

use crate::platform::directories::*;

/// All settings which are used by both, the client and the daemon
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Shared {
    pub secret: String,
    pub pueue_directory: String,
    pub use_unix_socket: bool,
    pub unix_socket_path: String,
    pub url: String,
    pub port: String,
    pub tls_enabled: bool,
    pub client_certificate: String,
}

/// All settings which are used by the client
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Client {
    pub read_local_logs: bool,
    pub show_confirmation_questions: bool,
    pub max_status_lines: Option<usize>,
}

/// All settings which are used by the daemon
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Daemon {
    pub default_parallel_tasks: usize,
    pub pause_on_failure: bool,
    pub callback: Option<String>,
    pub groups: HashMap<String, usize>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Settings {
    pub shared: Shared,
    pub client: Client,
    pub daemon: Daemon,
}

impl Settings {
    /// This function creates a new configuration instance and
    /// populates it with default values for every option.
    /// If a local config file already exists it is parsed and
    /// overwrites the default option values.
    /// The local config is located at "~/.config/pueue.yml".
    ///
    /// If `require_config` is `true`, an error will be thrown, if no configuration file can be found.
    pub fn new(require_config: bool, from_file: &Option<PathBuf>) -> Result<Settings> {
        let mut config = Config::new();

        let pueue_path = default_pueue_path()?;
        config.set_default("shared.secret", gen_random_secret())?;
        config.set_default("shared.pueue_directory", pueue_path.clone())?;
        config.set_default("shared.use_unix_socket", false)?;
        config.set_default("shared.unix_socket_path", get_unix_socket_path()?)?;
        config.set_default("shared.url", "localhost")?;
        config.set_default("shared.port", "6924")?;
        config.set_default("shared.tls_enabled", true)?;
        config.set_default("shared.client_certificate", pueue_path + "/client.cert")?;

        // Client specific config
        config.set_default("client.read_local_logs", true)?;
        config.set_default("client.show_confirmation_questions", false)?;
        config.set_default("client.max_status_lines", None::<i64>)?;

        // Daemon specific config
        config.set_default("daemon.default_parallel_tasks", 1)?;
        config.set_default("daemon.pause_on_failure", false)?;
        config.set_default("daemon.callback", None::<String>)?;
        config.set_default("daemon.groups", HashMap::<String, i64>::new())?;

        // Load the config from a very specific file path
        if let Some(path) = from_file {
            if !path.exists() {
                bail!("Couldn't find config at path {:?}", path);
            }
            info!("Using config file at: {:?}", path);
            let config_file = config::File::with_name(path.to_str().unwrap());
            config.merge(config_file)?;
        } else {
            // Load settings from the default config paths.
            parse_config(&mut config, require_config)?;
        }

        // Try to can deserialize the entire configuration
        Ok(config.try_into()?)
    }

    /// Try to read the config file without any default values.
    /// This is done by the daemon on startup.
    /// If the file can be read without any need for defaults, we don't have to persist it
    /// afterwards.
    pub fn read(require_config: bool, from_file: &Option<PathBuf>) -> Result<Settings> {
        let mut config = Config::new();

        // Load the config from a very specific file path
        if let Some(path) = from_file {
            if !path.exists() {
                bail!("Couldn't find config at path {:?}", path);
            }
            info!("Using config file at: {:?}", path);
            let config_file = config::File::with_name(path.to_str().unwrap());
            config.merge(config_file)?;
        } else {
            // Load settings from the default config paths.
            parse_config(&mut config, require_config)?;
        }

        // Try to can deserialize the entire configuration
        Ok(config.try_into()?)
    }

    /// Save the current configuration as a file to the configuration path.
    /// The file is written to the main configuration directory of the respective OS.
    pub fn save(&self, to_file: &Option<PathBuf>) -> Result<()> {
        let config_path = if let Some(path) = to_file {
            path.clone()
        } else {
            default_config_directory()?.join("pueue.yml")
        };
        let config_dir = config_path
            .parent()
            .ok_or_else(|| anyhow!("Couldn't resolve config dir"))?;

        // Create the config dir, if it doesn't exist yet
        if !config_dir.exists() {
            create_dir_all(config_dir)?;
        }

        let content = serde_yaml::to_string(self)?;
        let mut file = File::create(config_path)?;
        file.write_all(content.as_bytes())?;

        Ok(())
    }
}

/// Get all possible configuration paths and check if there are
/// configuration files at those locations.
/// All configs will be merged by importance.
///
/// If `require_config` is `true`, an error will be thrown, if no configuration file can be found.
fn parse_config(settings: &mut Config, require_config: bool) -> Result<()> {
    let mut config_found = false;
    info!("Parsing config files");
    for directory in get_config_directories()?.into_iter() {
        let path = directory.join("pueue.yml");
        info!("Checking path: {:?}", &path);
        if path.exists() {
            info!("Found config file at: {:?}", path);
            config_found = true;
            let config_file = config::File::with_name(path.to_str().unwrap());
            settings.merge(config_file)?;
        }
    }

    if require_config && !config_found {
        bail!("Couldn't find a configuration file. Did you start the daemon yet?");
    }

    Ok(())
}

/// Simple helper function to generate a random secret
fn gen_random_secret() -> String {
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                            abcdefghijklmnopqrstuvwxyz\
                            0123456789)(*&^%$#@!~";
    const PASSWORD_LEN: usize = 20;
    let mut rng = rand::thread_rng();

    let secret: String = (0..PASSWORD_LEN)
        .map(|_| {
            let idx = rng.gen_range(0, CHARSET.len());
            CHARSET[idx] as char
        })
        .collect();

    secret
}
