use serde::Deserialize;
use regex::Regex;
use html_escape::decode_html_entities;
use std::{thread, time};
use std::sync::{Arc, Mutex};


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

fn clean_html(text: &str, re: &Regex) -> String {
    // html unescape and regex replacement at once
    re.replace_all(&decode_html_entities(text), "").to_string()
}

fn check_keywords(sub: &str, com: &str, keywords: &[String]) -> Vec<String> {
    let mut matches = vec![];
    let full_text: String = format!("{} {}", sub, com).to_lowercase();
    for keyword in keywords {
        if full_text.contains(keyword) {
            matches.push(keyword.to_string());
        }
    }
    matches
}

fn check_board(board: &str, keywords: &[String], re: &Regex, added: &mut Vec<u32>) {
    let url = format!("https://a.4cdn.org/{}/catalog.json", board);
    let response = reqwest::blocking::get(url).unwrap();
    let deserlialize: Vec<Page> = response.json::<Vec<Page>>().unwrap();

    for page in deserlialize {
        for thread in page.threads {
            let number: u32 = thread.no;
            let thread_url = format!("https://boards.4chan.org/{}/thread/{}", board, number);
            let subject: String = match thread.sub {
                Some(sub) => clean_html(&sub, &re),
                None => String::from(""),
            };
            let comment: String = match thread.com {
                Some(com) => clean_html(&com, &re),
                None => String::from(""),
            };
            let matched_keywords: Vec<String> = check_keywords(&subject, &comment, keywords,) ;

            if !matched_keywords.is_empty() && !added.contains(&number){
                for keyword in matched_keywords {
                    println!("Keyword found: {} in {}", keyword, thread_url)
                }
                added.push(number);
            }
        }
    }
}



fn main() {
    let is_main_running: Arc<Mutex<bool>> = Arc::new(Mutex::new(true));
    let is_background_running: Arc<Mutex<bool>> = Arc::clone(&is_main_running);
    let background_monitor = thread::spawn(
        move || {
            let interval = time::Duration::from_secs(60);
            let re = Regex::new(r"<.*?>").unwrap();
            let keywords = [String::from("gentoo")];
            let mut added: Vec<u32> = vec![];
            while *is_background_running.lock().unwrap() {
                check_board("g", &keywords, &re, &mut added);
                thread::sleep(interval)
            }
        }
    );
    background_monitor.join().unwrap();
}
