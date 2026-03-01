use serde::Deserialize;
use std::{env, fs, path::PathBuf};

#[derive(Deserialize)]
#[serde(default)]
pub struct Config {}

impl Default for Config {
    fn default() -> Self {
        Self {}
    }
}

impl Config {
    pub fn load() -> Self {
        for dir in xdg_config_dirs() {
            let path = dir.join("red").join("config.toml");
            if let Ok(content) = fs::read_to_string(&path) {
                match toml::from_str(&content) {
                    Ok(config) => return config,
                    Err(e) => eprintln!("config error in {}: {e}", path.display()),
                }
            }
        }
        Config::default()
    }
}

fn xdg_config_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();

    // $XDG_CONFIG_HOME, defaults to ~/.config
    if let Ok(val) = env::var("XDG_CONFIG_HOME") {
        dirs.push(PathBuf::from(val));
    } else if let Ok(home) = env::var("HOME") {
        dirs.push(PathBuf::from(home).join(".config"));
    }

    // $XDG_CONFIG_DIRS, defaults to /etc/xdg
    if let Ok(val) = env::var("XDG_CONFIG_DIRS") {
        dirs.extend(val.split(':').map(PathBuf::from));
    } else {
        dirs.push(PathBuf::from("/etc/xdg"));
    }

    dirs
}
