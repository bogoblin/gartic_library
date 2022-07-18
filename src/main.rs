use std::ffi::OsString;
use std::fs;
use std::fs::{DirEntry};
use chrono::{DateTime, TimeZone, Utc};
use regex::Regex;

fn main() {
    let image_path = OsString::from("/media/bobby/Big/Pictures/Pictures Classic/Gartic Phone");
    let rounds = rounds_from_directory(image_path);
    for round in rounds {
        println!("{}", round.date);
    }
}

fn rounds_from_directory(directory_path: OsString) -> Vec<Round> {
    let date_re = Regex::new(r"^album_(\d{4})-(\d{2})-(\d{2})_(\d{2})-(\d{2})-(\d{2})").unwrap();
    let mut rounds = Vec::new();
    for entry in fs::read_dir(directory_path).unwrap() {
        let entry = entry.unwrap();
        let image_path = entry.file_name();
        // let image_path_str = image_path.clone();
        for cap in date_re.captures_iter(image_path.to_str().unwrap()) {
            let date = Utc.ymd((&cap[1]).parse().unwrap(), (&cap[2]).parse().unwrap(), (&cap[3]).parse().unwrap())
                .and_hms((&cap[4]).parse().unwrap(), (&cap[5]).parse().unwrap(), (&cap[6]).parse().unwrap());
            rounds.push(Round {
                image_path, date,
            });
            break;
        }
    }
    rounds.sort();
    return rounds;
}

#[derive(Eq, Ord, PartialEq, PartialOrd)]
struct Round {
    date: DateTime<Utc>,
    image_path: OsString,
}