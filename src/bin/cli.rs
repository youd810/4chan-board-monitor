use std::fs;
use board_monitor_4chan::{Config, config_path};
use board_monitor_4chan::config::display_boards;

fn main(){
    let config_path = config_path();
    if !config_path.exists() {
        let default_config = "interval = 60\n\n[[boards]]\nname = \"g\"\nkeywords = []\n";
        std::fs::write(&config_path, default_config).unwrap();
    }
    let read_config: String = fs::read_to_string(&config_path).expect("Failed to find config.toml");
    let mut config: Config = match toml::from_str(&read_config) {
        Ok(res) => res,
        Err(_) => {
            println!("Unable to fetch config from config.toml; make sure it's not empty and/or formatted properly");
            return;
        }
    };
    display_boards(&mut config, &config_path);
}