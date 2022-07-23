use std::{fs, thread};
use std::fs::File;
use std::io::Write;
use chrono::{Datelike, DateTime, Timelike, Utc};
use std::path::{Path, PathBuf};
use std::thread::JoinHandle;
use gif;
use png;

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

    // Output game html
    let mut html_page = File::create(game_dir.join("index.html")).unwrap();
    html_page.write(format!("<a href='..'>Back</a>\n").as_ref()).unwrap();
    for round_num in 1..i+1 {
        html_page.write(format!("<a href='{:02}'><img src='{:02}/round.gif'></a>\n", round_num, round_num).as_ref()).unwrap();
    }
}

fn rounds_from_directory(directory_path: &Path) -> Vec<Round> {
    let mut rounds = Vec::new();
    for entry in directory_path.read_dir().unwrap() {
        let entry = entry.unwrap();
        let file_date = entry.metadata().unwrap().modified().unwrap();
        let image_path = entry.path();
        let date = DateTime::from(file_date);
        rounds.push(Round { image_path, date });
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
        // Split gif into separate frames
        fs::create_dir_all(round_dir.as_path()).unwrap();
        let file = File::open(&self.image_path).unwrap();
        let mut options = gif::DecodeOptions::new();
        options.set_color_output(gif::ColorOutput::RGBA);
        println!("Processing {}", &self.image_path.to_str().unwrap());
        let mut decoder = options.read_info(&file).unwrap();
        let mut i = 0;
        while let Some(frame) = decoder.read_next_frame().unwrap() {
            i += 1;
            let mut image = File::create(round_dir.join(format!("{:02}.png", i))).unwrap();
            let mut encoder = png::Encoder::new(
                &mut image,
                frame.width as u32,
                frame.height as u32,
            );
            encoder.set_color(png::ColorType::Rgba);
            let mut writer = encoder.write_header().unwrap();
            writer.write_image_data(Vec::from(frame.buffer.clone()).as_mut_slice()).unwrap();
        }

        // Generate HTML page
        let mut html_page = File::create(round_dir.join("index.html")).unwrap();
        html_page.write(format!("<a href='..'>Back</a>\n").as_ref()).unwrap();
        for round_num in 1..i+1 {
            html_page.write(format!("<figure><img src='{:02}.png'></figure>\n", round_num).as_ref()).unwrap();
        }

        // Copy original gif
        fs::copy(&self.image_path, round_dir.join("round.gif")).unwrap();
    }
}

struct Game {
    rounds: Vec<Round>,
}

impl Game {
    fn game_date(&self) -> DateTime<Utc> {
        self.rounds.get(0).unwrap().date
    }

    fn dir(&self) -> PathBuf {
        let first_date = self.game_date();
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