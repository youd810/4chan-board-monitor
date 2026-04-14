use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::{env, fs, io, thread, time};

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

fn clear_screen(){
    if cfg!(target_os = "windows") {
        std::process::Command::new("cmd").args(["/c", "cls"]).status().unwrap();
    } else {
        std::process::Command::new("clear").status().unwrap();
    }
}

fn read_input() -> String {
    let mut input = String::new();
    println!("");
    io::stdin().read_line(&mut input).expect("Failed to read input");
    input.trim().to_string()
}

fn read_num() -> Option<usize> {
    // usize for indexing
    read_input().trim().parse::<usize>().ok()
}

pub fn display_boards(config: &mut Config, path: &PathBuf) {
    loop {
        clear_screen();
        println!("Input the number attached to the board to navigate into said board");
        println!("");
        println!("Added Boards: ");
        println!("");
        for (i, board) in config.boards.iter().enumerate() {
            println!("{}. /{}/", i+1, board.name);       
        }
        println!("");
        println!("Interval: {} seconds", config.interval);
        println!("");
        println!("{}. Add a board", config.boards.len() + 1);
        println!("{}. Delete a board", config.boards.len() + 2);
        println!("{}. Edit interval", config.boards.len() + 3);
        println!("{}. Exit", config.boards.len() + 4);
        match read_num() {
            Some(num) if num >= 1 && num <= config.boards.len() => display_keywords(config, num - 1, path),
            Some(num) if num == (config.boards.len() + 1) => add_board(config, path),
            Some(num) if num == (config.boards.len() + 2) => delete_board(config, path),
            Some(num) if num == (config.boards.len() + 3) => edit_interval(config, path),
            Some(num) if num == (config.boards.len() + 4) => {
                clear_screen();
                std::process::exit(0);
            },
            _ => {
                println!("Invalid input!");
                thread::sleep( time::Duration::from_millis(250));
                continue;
            },
        }
    }
}

fn add_board(config: &mut Config, path: &PathBuf){
    loop {
        clear_screen();
        for board in config.boards.iter() {
            println!("{}", board.name);       
        }
        println!("");
        println!("Input a board name ('g', 'v', etc. without the quotation marks)");
        println!("input 'back' to go back");
        let input: String = read_input();
        let boards: [&str; 76] = [
            "a", "aco", "adv", "an", "b", "bant", "biz", "c", "cgl", "ck", "cm", "co", "d", "diy",
            "e", "f", "fit", "g", "gd", "gif", "h", "hc", "his", "hm", "hr", "i", "ic", "int", "jp",
            "k", "lgbt", "lit", "m", "mlp", "mu", "n", "news", "o", "out", "p", "po", "pol", "pw",
            "qa", "qst", "r", "r9k", "s", "s4s", "sci", "soc", "sp", "t", "tg", "toy", "trash",
            "trv", "tv", "u", "v", "vg", "vip", "vm", "vmg", "vp", "vr", "vrpg", "vst", "w", "wg",
            "wsg", "wsr", "x", "xs", "y", "3",
        ];
        if boards.contains(&input.as_str()) {
            // we push a struct since the content of boards is a vector of structs 
            config.boards.push( Board{
                name: input,
                keywords: Vec::new(), 
            });
            save_config(config, path);
        } else if input == "back" {
            break;
        } else {
            println!("Invalid input!");
            thread::sleep( time::Duration::from_millis(250));
            continue;
        }
    }
}

fn delete_board(config: &mut Config, path: &PathBuf) {
    loop {
        clear_screen();
        for (i, board) in config.boards.iter().enumerate() {
            println!("{}. /{}/", i+1, board.name);  
        }
        println!("");
        println!("{}. Back", config.boards.len() + 1);
        println!("");
        println!("Input the number attached to the board");
        match read_num() {
            Some(num) if num >= 1 && num <= config.boards.len() => {
                config.boards.remove(num - 1);
                save_config(config, path);
            },
            Some(num) if num == (config.boards.len() + 1) => break,
            _ => {
                println!("Invalid input!");
                thread::sleep( time::Duration::from_millis(250));
                continue;
            }
        };
    }
}

fn edit_interval(config: &mut Config, path: &PathBuf) {
    loop {
        clear_screen();
        println!("Set interval between request (2 seconds minimum)");
        println!("(this doesn't include the hardcoded interval for checking each board, which is 1.1 second)");
        println!("(current interval: {})", config.interval);
        println!("");
        println!("Input '0' to go back");
        match read_num() {
            Some(num) if num >= 2 => {
                config.interval = num as u64;
                save_config(config, path);
            },
            Some(num) if num < 2 && num != 0 => {
                println!("Minimum interval of 2 seconds");
                thread::sleep( time::Duration::from_millis(250));
                continue;
            },
            Some(num) if num == 0 => break,
            _ => { 
                println!("Invalid input!");
                thread::sleep( time::Duration::from_millis(250));
                continue;
            },
        }
    }
}

fn display_keywords(config: &mut Config, board_idx: usize, path: &PathBuf) {
    loop {
        clear_screen();
        println!("Added Keywords for /{}/:", config.boards[board_idx].name);
        println!("");
        for keyword in config.boards[board_idx].keywords.iter() {
            println!("{}", keyword);
        }
        println!("");
        println!("1. Add a keyword");
        println!("2. Delete a keyword");
        println!("3. Back");
        match read_num() {
            Some(num) if num == 1 => add_keyword(config, board_idx, path),
            Some(num) if num == 2 => delete_keyword(config, board_idx, path),
            Some(num) if num == 3 => break,
            _ => {
                println!("Invalid input!");
                thread::sleep( time::Duration::from_millis(250));
                continue
            }
        }
    }
}

fn add_keyword(config: &mut Config, board_idx: usize, path: &PathBuf) {
    loop {
        clear_screen();
        for keyword in config.boards[board_idx].keywords.iter() {
            println!("{}", keyword);
        }
        println!("");
        println!("Input a keyword (multiple words are allowed)");
        println!("input '0' to go back");
        let input: String = read_input();
        if input == "0" {
            break;
        } else if input == "" {
            println!("Invalid input!");
            thread::sleep( time::Duration::from_millis(250));
            continue;
        } else {
            config.boards[board_idx].keywords.push(input);
            save_config(config, path);
        }
    }
}

fn delete_keyword(config: &mut Config, board_idx: usize, path: &PathBuf) {
    loop {
        clear_screen();
        for (i, keyword) in config.boards[board_idx].keywords.iter().enumerate() {
            println!("{}. {}", i+1, keyword);
        }
        println!("");
        println!("{}. Back", config.boards[board_idx].keywords.len() + 1);
        println!("");
        println!("Input the number attached to the keyword");
        match read_num() {
            Some(num) if num >= 1 && num <= config.boards[board_idx].keywords.len() => {
                config.boards[board_idx].keywords.remove(num - 1);
                save_config(config, path);
                println!("Config saved")
            },
            Some (num) if num == (config.boards[board_idx].keywords.len() + 1) => break,
            _ => {
                println!("Invalid input!");
                thread::sleep( time::Duration::from_millis(250));
                continue;
            },
        };
    }
}

fn save_config(config: &mut Config, path: &PathBuf) {
    let serialize = toml::to_string(config).unwrap();
    fs::write(path, serialize).expect("Failed to save config");
    println!("Config saved");
    thread::sleep( time::Duration::from_millis(500));
}