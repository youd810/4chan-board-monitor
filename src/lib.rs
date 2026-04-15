pub mod config;

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::env;

// (conv text to sruct, struct to text)
#[derive(Deserialize, Serialize)]
pub struct Board {
    pub name: String,
    pub keywords: Vec<String> 
}

#[derive(Deserialize, Serialize)]
pub struct Config {
    pub interval: u64,
    pub boards: Vec<Board>
}

pub fn config_path() -> PathBuf {
    let conf_path = if std::path::Path::new("config.toml").exists() {
        std::path::PathBuf::from("config.toml")
    } else {
        // gets the aboslute path no matter where the program is started if relative returns an err
        let mut path = env::current_exe().unwrap();
        // removes the .exe from the path, then adds the config filename
        path.pop();
        path.push("config.toml");
        path
    };
    conf_path
}