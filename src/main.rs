use std::fs;
use std::fs::{DirEntry};
use chrono::DateTime;
use regex::Regex;

fn main() {
    let image_path = "/media/bobby/Big/Pictures/Pictures Classic/Gartic Phone";
    let date_re = Regex::new(r"^album_(\d{4})-(\d{2})-(\d{2})_(\d{2})-(\d{2})-(\d{2})").unwrap();
    for entry in fs::read_dir(image_path).unwrap() {
        let entry = entry.unwrap();
        let file_name = entry.file_name();
        for cap in date_re.captures_iter(file_name.to_str().unwrap()) {
            println!("{}, {}, {}, {}, {}, {}", &cap[1], &cap[2], &cap[3], &cap[4], &cap[5], &cap[6]);
        }
        // println!("{}", entry.file_name().to_str().unwrap());
    }
}
