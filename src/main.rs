use serde::Deserialize;
use regex::Regex;
use html_escape::decode_html_entities;
use std::collections::HashSet;
use std::{thread, time, env, io};
use std::sync::{Arc, Mutex};
use notify_rust::Notification;
use tray_icon::{TrayIconBuilder, menu::{Menu, MenuItem, MenuEvent}};
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{EventLoop, ActiveEventLoop, ControlFlow};
use winit::window::WindowId;

#[derive(Deserialize)]
struct Board {
    name: String,
    keywords: Vec<String> 
}

#[derive(Deserialize)]
struct Config {
    interval: u64,
    boards: Vec<Board>
}

#[derive(Deserialize, Debug)]
// parse the reqwest json with struct
struct Page {
    threads: Vec<Thread>
}

#[derive(Deserialize, Debug)]
struct Thread {
    no: u32,
    sub: Option<String>,
    com: Option<String>,
}

struct App {
    is_running: Arc<Mutex<bool>>,
    quit_id: tray_icon::menu::MenuId,
}

impl ApplicationHandler for App {
    // these empty functions are here to satisfy the compiler
    fn resumed(&mut self, _event_loop: &ActiveEventLoop){}
    fn window_event(&mut self, _event_loop: &ActiveEventLoop, _id: WindowId, _event: WindowEvent) {}
    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        // checks MenuEvent for an event
        if let Ok(event) = MenuEvent::receiver().try_recv() {
            // if it's true and it's from quit_item, the program will close
            if event.id == self.quit_id {
                *self.is_running.lock().unwrap() = false;
                event_loop.exit();
            }
        }
    }
}


fn clean_html(text: &str, re: &Regex) -> String {
    // html unescape and regex replacement at once
    re.replace_all(&decode_html_entities(text), "").to_string()
}

fn check_keywords(sub: &str, com: &str, keywords: &[String]) -> Vec<String> {
    let full_text: String = format!("{} {}", sub, com).to_lowercase();
    let matches: Vec<String> = keywords
        .iter()
        .filter(|keyword: &&String| full_text.contains(*keyword))
        .cloned()
        .collect();
    matches
}

fn error_notif<T>(e: T) where T: std::fmt::Display {
    Notification::new()
        .summary("4chan Monitor")
        .body(&format!("ERROR: {}", e))
        .sound_name("Default")
        .timeout(notify_rust::Timeout::Milliseconds(5000))
        .show()
        .unwrap();
}

fn check_board(board: &str, keywords: &[String], re: &Regex, checked: &mut HashSet<u32>) {
    let url: String = format!("https://a.4cdn.org/{}/catalog.json", board);
    let response = match reqwest::blocking::get(url) {
        Ok(res) => res,
        Err(e) => {
            error_notif(e);
            return;
        }
    };
    let deserialize: Vec<Page> = match response.json::<Vec<Page>>() {
        Ok(res) => res,
        Err(e) => {
            error_notif(e);
            return;
        }
    };

    for page in deserialize {
        for thread in page.threads {
            let number: u32 = thread.no;
            
            if checked.contains(&number) {
                continue;
            }

            let thread_url: String = format!("https://boards.4chan.org/{}/thread/{}", board, number);
            let subject: String = match thread.sub {
                Some(sub) => clean_html(&sub, re),
                None => String::from(""),
            };
            let comment: String = match thread.com {
                Some(com) => clean_html(&com, re),
                None => String::from(""),
            };
            let matched_keywords: Vec<String> = check_keywords(&subject, &comment, keywords,) ;

            if !matched_keywords.is_empty() {
                for keyword in matched_keywords {
                    Notification::new()
                        .summary("4chan Monitor")
                        .body(&format!("Keyword found: {} in {}", keyword, thread_url))
                        .sound_name("Default")
                        .timeout(notify_rust::Timeout::Milliseconds(5000))
                        .show()
                        .unwrap();
                } 
            }
            checked.insert(number);
        }
    }
}

fn create_icon() -> tray_icon::Icon {
    let bytes = include_bytes!("../assets/icon.png");
    let img = image::load_from_memory(bytes).unwrap().to_rgba8();
    let (w, h) = img.dimensions();
    tray_icon::Icon::from_rgba(img.into_raw(), w, h).unwrap()
}

fn read_input() -> String {
    let mut input = String::new();
    io::stdin().read_line(&mut input).expect("Failed to read input");
    input.trim().to_string()
}

fn read_num() -> Option<usize> {
    read_input().trim().parse::<usize>().ok()
}

fn display_boards(config: &mut Config) {
    loop {
        for (i, board) in config.boards.iter().enumerate() {
            println!("{}. {}", i+1, board.name);       
        }
        println!("");
        println!("{}. Add a board", config.boards.len() + 1);
        println!("{}. Delete a board", config.boards.len() + 2);
        match read_num() {
            Some(num) if num >= 1 && num <= config.boards.len() => display_keywords(config, num - 1),
            Some(num) if num == (config.boards.len() + 1) => add_board(config),
            Some(num) if num == (config.boards.len() + 2) => delete_board(config),
            _ => {
                println!("Invalid input!");
                continue;
            },
        }
    }
}

fn add_board(config: &mut Config){
    loop {
        for board in config.boards.iter() {
            println!("{}", board.name);       
        }
        println!("");
        println!("Input a board name ('g', 'v', etc. without the quotation mark)");
        println!("input 'back' to go back");
        let input = read_input();
        let boards = [
            "a", "aco", "adv", "an", "b", "bant", "biz", "c", "cgl", "ck", "cm", "co", "d", "diy",
            "e", "f", "fit", "g", "gd", "gif", "h", "hc", "his", "hm", "hr", "i", "ic", "int", "jp",
            "k", "lgbt", "lit", "m", "mlp", "mu", "n", "news", "o", "out", "p", "po", "pol", "pw",
            "qa", "qst", "r", "r9k", "s", "s4s", "sci", "soc", "sp", "t", "tg", "toy", "trash",
            "trv", "tv", "u", "v", "vg", "vip", "vm", "vmg", "vp", "vr", "vrpg", "vst", "w", "wg",
            "wsg", "wsr", "x", "xs", "y", "3"
        ];
        if boards.contains(&input.as_str()) {
            // how do i actually add the board?
        } else if input == "back" {
            break;
        } else {
            println!("Invalid input!");
            continue;
        }
    }
}

fn delete_board(config: &mut Config) {
    loop {
        for (i, board) in config.boards.iter().enumerate() {
            println!("{}. {}", i+1, board.name);  
        }
        println!("");
        println!("{}. Back", config.boards.len() + 1);
        match read_num() {
            Some(num) if num >= 1 && num <= config.boards.len() => {}, // delete board 
            Some(num) if num == (config.boards.len() + 1) => break,
            _ => {
                println!("Invalid input!");
                continue;
            }
        }
    }
}


fn display_keywords(config: &mut Config, board_idx: usize) {
    loop {
        for keyword in config.boards[board_idx].keywords.iter() {
            println!("{}", keyword);
        }
        println!("");
        println!("1. Add a keyword");
        println!("2. Delete a keyword");
        println!("3. Back");
        match read_num() {
            Some(num) if num == 1 => add_keyword(config, board_idx),
            Some(num) if num == 2 => delete_keyword(config, board_idx),
            Some(num) if num == 3 => break,
            _ => {
                println!("Invalid input!");
                continue
            }
        }
    }
}

fn add_keyword(config: &mut Config, board_idx: usize) {
    loop {
        for keyword in config.boards[board_idx].keywords.iter() {
            println!("{}", keyword);
        }
        println!("");
        println!("Input a keyword (one word per input)");
        println!("input '0' to go back");
        let input = read_input();
        if input == "0" {
            break;
        } else if input.contains(" "){
            println!("1 word at a time please");
            continue;
        } else {
            // add the keyword
        }
    }
}

fn delete_keyword(config: &mut Config, board_idx: usize) {
    loop {
        for (i, keyword) in config.boards[board_idx].keywords.iter().enumerate() {
            println!("{}. {}", i, keyword);
        }
        println!("");
        println!("{}. Back", config.boards[board_idx].keywords.len() + 1);
        println!("Input the number of the word you wish to be deleted");
        match read_num() {
            Some(num) if num >= 1 && num <= config.boards[board_idx].keywords.len() => {},
            Some (num) if num == (config.boards[board_idx].keywords.len() + 1) => break,
            _ => {
                println!("Invalid input!");
                continue;
            },
        }
    }
}

fn main() {
    let config_path = if std::path::Path::new("config.toml").exists() {
            std::path::PathBuf::from("config.toml")
        } else {
            // gets the aboslute path no matter where the program is started if relative returns an err
            let mut path = std::env::current_exe().unwrap();
            // removes the .exe from the path, then adds the config filename
            path.pop();
            path.push("config.toml");
            path
        };
    let read_config: String = std::fs::read_to_string(&config_path).expect("Failed to find config.toml");
    let mut config: Config = match toml::from_str(&read_config) {
        Ok(res) => res,
        Err(_) => {
            error_notif("Unable to fetch config from config.toml; make sure it's not empty and/or formatted properly");
            return;
        }
    };

    let args: Vec<String> = std::env::args().collect();

    if args.len() > 1 && args[1] == "config" {
        display_boards(&mut config);
    } 

    else {
        let is_main_running: Arc<Mutex<bool>> = Arc::new(Mutex::new(true));
        let is_background_running: Arc<Mutex<bool>> = Arc::clone(&is_main_running);
        let _background_monitor = thread::spawn(
            move || {
                let interval = time::Duration::from_secs(config.interval);
                let re: Regex = Regex::new(r"<.*?>").unwrap();
                // hashset for 0(1) lookup speed
                let mut checked: HashSet<u32> = HashSet::new();

                let mut init_timestamp = std::fs::metadata(&config_path).unwrap().modified().unwrap();

                while *is_background_running.lock().unwrap() {
                    let current_timestamp = std::fs::metadata(&config_path).unwrap().modified().unwrap();
                    
                    // detects changes in the config file
                    if init_timestamp != current_timestamp {
                        // this var will die after this if statement (cont)
                        let new_read_config = std::fs::read_to_string(&config_path).expect("Failed to find config.toml");

                        // (cont) but config will keep its value because of owned String
                        match toml::from_str(&new_read_config) {
                            // reassign config ONLY if the toml parse returns Ok
                            Ok(res) => config = res,
                            Err(_) => {
                                error_notif("Unable to fetch config from config.toml; changes will be ignored until restart. Please fix the issue by then");
                            }
                        };
                        init_timestamp = current_timestamp
                    }

                    for board in config.boards.iter() {
                        check_board(&board.name, &board.keywords, &re, &mut checked);
                        thread::sleep(time::Duration::from_secs(2))
                    }
                    thread::sleep(interval)
                }
            }
        );

        // new(text, clickable, kb shortcut)
        let quit_item = MenuItem::new("Quit", true, None);
        let menu = Menu::new();
        menu.append(&quit_item).unwrap();
        let _tray_icon = TrayIconBuilder::new()
            .with_menu(Box::new(menu))
            .with_tooltip("4chan Monitor")
            .with_icon(create_icon())
            .build()
            .unwrap();
        
        let event_loop = EventLoop::new().unwrap();
        event_loop.set_control_flow(ControlFlow::Poll);
        let mut app = App {
            // binds the main thread to the event loop, which will kill the program in its stead
            is_running: is_main_running,
            quit_id: quit_item.id().clone(),
        };

        event_loop.run_app(&mut app).unwrap();
    }
}
