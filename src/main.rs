use std::{fs, thread};
use std::cmp::Ordering;
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

    fs::copy(Path::new("./style.css"), output_path.join("style.css")).unwrap();

    let mut html_page = html_page(output_path.join("index.html").as_path(), Path::new("."));
    html_page.write("<h1>Gartic Phone Games</h1>".as_ref()).unwrap();
    html_page.write("<section class='games'>".as_ref()).unwrap();
    for game in games {
        let mut handles = vec![];
        let mut preview_images = String::new();
        for round_num in 1..=game.rounds.len() {
            preview_images.push_str(format!("<img src='{}/{:02}/01.png'>", game.dir().to_str().unwrap(),  round_num).as_ref());
        }
        html_page.write(format!("<a class='game' href='{}'>\
        <div class='date'>{}</div>\
        {}\
        </a>", game.dir().to_str().unwrap(), game.game_date(), preview_images.as_str()).as_ref()).unwrap();

        output_game(&output_path, game, &mut handles);
        for handle in handles {
            handle.join().unwrap();
        }
    }
    html_page.write("</section>".as_ref()).unwrap();
}

fn output_game(output_dir: &Path, game: Game, handles: &mut Vec<JoinHandle<()>>) {
    let game_dir = output_dir.join(game.dir());
    fs::create_dir_all(game_dir.as_path()).unwrap();
    let num_rounds = game.rounds.len();

    // Output game html
    let mut html_page = html_page(game_dir.join("index.html").as_path(), Path::new("../../../.."));
    html_page.write(format!("<a href='../../../..'>Back</a>\n").as_ref()).unwrap();
    html_page.write(format!("<h1 class='date'>{}</h1>", game.game_date()).as_ref()).unwrap();
    html_page.write("<section class='rounds'>\n".as_ref()).unwrap();
    for round_num in 1..=num_rounds {
        html_page.write(format!("<a class='round' href='{:02}'><img src='{:02}/round.gif'></a>\n", round_num, round_num).as_ref()).unwrap();
    }
    html_page.write("</section>\n".as_ref()).unwrap();

    for (i, round) in game.rounds.into_iter().enumerate() {
        let round_dir = game_dir.join(format!("{:02}", i+1));
        handles.push(thread::spawn(move || {
            round.output_images(round_dir);
        }));
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
    games.sort();
    games.reverse();
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
        let mut html_page = html_page(round_dir.join("index.html").as_path(), Path::new("../../../../.."));
        html_page.write(format!("<a href='..'>Back</a>\n").as_ref()).unwrap();
        html_page.write("<section class='frames'>\n".as_ref()).unwrap();
        for frame_num in 1..i+1 {
            html_page.write(format!("<figure class='frame'><img src='{:02}.png'></figure>\n", frame_num).as_ref()).unwrap();
        }
        html_page.write("</section>\n".as_ref()).unwrap();

        // Copy original gif
        fs::copy(&self.image_path, round_dir.join("round.gif")).unwrap();
    }
}

#[derive(Eq, Clone)]
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

impl PartialEq<Self> for Game {
    fn eq(&self, other: &Self) -> bool {
        self.game_date() == other.game_date()
    }
}

impl PartialOrd<Self> for Game {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Game {
    fn cmp(&self, other: &Self) -> Ordering {
        self.game_date().cmp(&other.game_date())
    }
}

fn html_page(page_path: &Path, root_path: &Path) -> File {
    let mut file = File::create(page_path).unwrap();
    file.write("<head>\n".as_ref()).unwrap();
    file.write(format!("<link type='text/css' rel='stylesheet' href='{}/style.css'>\n", root_path.to_str().unwrap()).as_ref()).unwrap();
    file.write("<title>Gartic Phone Library</title>".as_ref()).unwrap();
    file.write("</head>\n".as_ref()).unwrap();
    file.write("<body>\n".as_ref()).unwrap();

    file
}