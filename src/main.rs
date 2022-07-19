use std::{fs, thread};
use std::fs::File;
use chrono::{Datelike, DateTime, Timelike, TimeZone, Utc};
use regex::Regex;
use std::path::{Path, PathBuf};
use std::thread::JoinHandle;
use gif::{Frame};

fn main() {
    let image_path = Path::new("/media/bobby/Big/Pictures/Pictures Classic/Gartic Phone");
    let output_path = Path::new("/media/bobby/Big/Projects/Gartic Phone Static");
    let rounds = rounds_from_directory(image_path);
    let games = games_from_rounds(rounds);

    let mut handles = vec![];
    for game in games {
        output_game(&output_path, game, &mut handles);
    }
    for handle in handles {
        handle.join().unwrap();
    }
}

fn output_game(output_dir: &Path, game: Game, handles: &mut Vec<JoinHandle<()>>) {
    let game_dir = output_dir.join(game.dir());
    let mut i = 0;
    for round in game.rounds {
        i+=1;
        let round_dir = game_dir.join(format!("{:02}", i));
        handles.push(thread::spawn(move || {
            round.output_images(round_dir);
        }));
    }
}

fn rounds_from_directory(directory_path: &Path) -> Vec<Round> {
    let date_re = Regex::new(r"album_(\d{4})-(\d{2})-(\d{2})_(\d{2})-(\d{2})-(\d{2})\.gif$").unwrap();
    let mut rounds = Vec::new();
    for entry in directory_path.read_dir().unwrap() {
        let entry = entry.unwrap();
        let image_path = entry.path();
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
    image_path: PathBuf,
}

impl Round {
    fn output_images(&self, round_dir: PathBuf) {
        let palette = &self.get_palette();

        fs::create_dir_all(round_dir.as_path()).unwrap();
        let file = File::open(&self.image_path).unwrap();
        let mut options = gif::DecodeOptions::new();
        options.set_color_output(gif::ColorOutput::RGBA);
        println!("Processing {}", &self.image_path.to_str().unwrap());
        let mut decoder = options.read_info(&file).unwrap();
        let mut i = 0;
        while let Some(frame) = decoder.read_next_frame().unwrap() {
            let mut image = File::create(round_dir.join(format!("{:02}.gif", i+1))).unwrap();
            i += 1;
            let mut encoder = gif::Encoder::new(
                &mut image,
                frame.width,
                frame.height,
                &palette[..],
            ).unwrap();
            let new_frame = Frame::from_rgba(frame.width, frame.height, Vec::from(frame.buffer.clone()).as_mut_slice());
            encoder.write_frame(&new_frame).unwrap();
        }
    }

    fn get_palette(&self) -> Vec<u8> {
        let file = File::open(&self.image_path).unwrap();
        let mut options = gif::DecodeOptions::new();
        options.set_color_output(gif::ColorOutput::RGBA);
        let decoder = options.read_info(&file).unwrap();
        Vec::from(decoder.global_palette().unwrap())
    }
}

struct Game {
    rounds: Vec<Round>,
}

impl Game {
    fn dir(&self) -> PathBuf {
        let first_date = self.rounds.get(0).unwrap().date;
        PathBuf::from(format!("{:04}/{:02}/{:02}/{:02}{:02}{:02}",
                              first_date.year(),
                              first_date.month(),
                              first_date.day(),
                              first_date.hour(),
                              first_date.minute(),
                              first_date.second(),
        ))
    }
}