use std::ffi::OsString;
use std::fs;
use chrono::{DateTime, TimeZone, Utc};
use regex::Regex;

fn main() {
    let image_path = OsString::from("/media/bobby/Big/Pictures/Pictures Classic/Gartic Phone");
    let rounds = rounds_from_directory(image_path);
    // for round in rounds {
    //     println!("{}", round.date);
    // }
    let games = games_from_rounds(rounds);
    for game in games {
        println!("{} {} rounds", game.rounds.get(0).unwrap().image_path.to_str().unwrap(), game.rounds.len());
    }
}

fn rounds_from_directory(directory_path: OsString) -> Vec<Round> {
    let date_re = Regex::new(r"album_(\d{4})-(\d{2})-(\d{2})_(\d{2})-(\d{2})-(\d{2})\.gif$").unwrap();
    let mut rounds = Vec::new();
    for entry in fs::read_dir(directory_path).unwrap() {
        let entry = entry.unwrap();
        let image_path = entry.path().into_os_string();
        for cap in date_re.captures_iter(image_path.to_str().unwrap()) {
            let date = Utc.ymd((&cap[1]).parse().unwrap(), (&cap[2]).parse().unwrap(), (&cap[3]).parse().unwrap())
                .and_hms((&cap[4]).parse().unwrap(), (&cap[5]).parse().unwrap(), (&cap[6]).parse().unwrap());
            rounds.push(Round { image_path, date });
            break;
        }
    }
    rounds.sort();
    return rounds;
}

fn games_from_rounds(rounds: Vec<Round>) -> Vec<Game> {
    let minutes_gap = 10;
    let mut game_indices = Vec::new();
    let first_round = rounds.get(0).expect("No rounds");
    let mut prev_date = first_round.date;
    for (i, round) in rounds.iter().enumerate() {
        let time_difference = round.date - prev_date;
        if time_difference.num_minutes() > minutes_gap {
            game_indices.push(i);
        }
        prev_date = round.date;
    }
    game_indices.push(rounds.len());
    let mut last = 0;
    let mut games = Vec::new();
    for i in game_indices {
        games.push(Game{rounds: rounds[last..i].to_vec()});
        last = i;
    }
    return games;
}

#[derive(Eq, Ord, PartialEq, PartialOrd, Clone)]
struct Round {
    date: DateTime<Utc>,
    image_path: OsString,
}

struct Game {
    rounds: Vec<Round>,
}