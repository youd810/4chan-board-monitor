use serde::Deserialize;
use regex::Regex;
use html_escape::decode_html_entities;
use std::collections::HashSet;
use std::{thread, time};
use std::sync::{Arc, Mutex};
use notify_rust::Notification;
use tray_icon::{TrayIconBuilder, menu::{Menu, MenuItem}};
use tray_icon::menu::MenuEvent;
use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{EventLoop, ActiveEventLoop, ControlFlow};
use winit::window::WindowId;

#[derive(Deserialize)]
// WIP
struct Board {
    name: String,
    keywords: Vec<String> 
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
    let mut matches: Vec<String> = vec![];
    let full_text: String = format!("{} {}", sub, com).to_lowercase();
    for keyword in keywords {
        if full_text.contains(keyword) {
            matches.push(keyword.to_string());
        }
    }
    matches
}

fn error_notif<T>(e: T) where T: std::fmt::Display {
    Notification::new()
        .summary("4chan Monitor")
        .body(&format!("Something is wrong: {}", e))
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
    let deserlialize: Vec<Page> = match response.json::<Vec<Page>>() {
        Ok(res) => res,
        Err(e) => {
            error_notif(e);
            return;
        }
    };

    for page in deserlialize {
        for thread in page.threads {
            let number: u32 = thread.no;
            
            if checked.contains(&number) {
                continue;
            }

            let thread_url: String = format!("https://boards.4chan.org/{}/thread/{}", board, number);
            let subject: String = match thread.sub {
                Some(sub) => clean_html(&sub, &re),
                None => String::from(""),
            };
            let comment: String = match thread.com {
                Some(com) => clean_html(&com, &re),
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

fn main() {
    let is_main_running: Arc<Mutex<bool>> = Arc::new(Mutex::new(true));
    let is_background_running: Arc<Mutex<bool>> = Arc::clone(&is_main_running);
    let _background_monitor = thread::spawn(
        move || {
            let interval = time::Duration::from_secs(60);
            let re = Regex::new(r"<.*?>").unwrap();
            // this is for testing; remove later
            let keywords = [String::from("gentoo")];
            // hashset for 0(1) lookup speed
            let mut checked: HashSet<u32> = HashSet::new();
            while *is_background_running.lock().unwrap() {
                check_board("g", &keywords, &re, &mut checked);
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
        // bind the main thread to the event loop and kill the program in its stead
        is_running: is_main_running,
        quit_id: quit_item.id().clone(),
    };

    event_loop.run_app(&mut app).unwrap();
}
