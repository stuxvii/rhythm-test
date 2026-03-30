use raylib::prelude::*;
use serde::Deserialize;
use std::env;
use std::fs;
use std::path::Path;

const PROGRAM_NAME: &str = "Rhythm";
const LANE_WIDTH: i32 = 100;
const NOTE_HEIGHT: f32 = 20.;
const RECEPTOR_Y: f32 = 460.;

const TIME_FLAWLESS: f32 = 0.025;
const TIME_PERFECT: f32 = 0.100;
const TIME_NICE: f32 = 0.250;
const TIME_MISS: f32 = 0.5;

fn draw_text_shadow(d: &mut RaylibDrawHandle, text: String, height: i32) {
    let new_height = height * 16;
    d.draw_text(&text, 1, new_height, 10, Color::BLACK);
    let new_height = new_height + 1;
    d.draw_text(&text, 0, new_height, 10, Color::WHITE);
}

#[derive(Debug, Clone, Copy, PartialEq, Deserialize, Default)]
#[serde(rename_all = "PascalCase")]
enum Judgment {
    Flawless,
    Perfect,
    Nice,
    Okay,
    Miss,
    #[default]
    None,
}

impl Judgment {
    fn string(j: Judgment) -> String {
        match j {
            Judgment::Flawless => String::from("FLAWLESS!!!"),
            Judgment::Perfect => String::from("PERFECT!!!"),
            Judgment::Nice => String::from("NICE!!!"),
            Judgment::Okay => String::from("OK"),
            Judgment::Miss => String::from("MISS! FUCKER!"),
            Judgment::None => String::from(""),
        }
    }

    fn get_judgment(time_diff: f32) -> Judgment {
        if time_diff <= TIME_FLAWLESS {
            return Judgment::Flawless;
        } else if time_diff <= TIME_PERFECT {
            return Judgment::Perfect;
        } else if time_diff <= TIME_NICE {
            return Judgment::Nice;
        } else if time_diff <= TIME_MISS {
            return Judgment::Okay;
        } else if time_diff >= TIME_MISS {
            return Judgment::Miss;
        } else {
            return Judgment::None
        }
    }
}

#[derive(Debug, Deserialize)]
struct SongData {
    pub bpm: f32,
    pub song: String,
    pub offset: f32,
    pub notes: Vec<Note>,
}

#[derive(Debug, Deserialize)]
struct Note {
    pub lane: u8,
    pub time: f32,

    #[serde(default)]
    accuracy: f32,
    #[serde(default)]
    state: Judgment,
    #[serde(default)]
    empty: bool,
}

fn check_note_hit(notes: &mut [Note], lane: u8, current_time: f32) -> Judgment {
    if let Some(note) = notes.iter_mut().find(|n| {
        n.lane == lane && n.state == Judgment::None && (n.time - current_time).abs() <= TIME_MISS
    }) {
        let time_diff = (note.time - current_time).abs();
        note.state = Judgment::get_judgment(time_diff);
        note.accuracy = time_diff;

        return note.state;
    }
    Judgment::None
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", PROGRAM_NAME);
    let args: Vec<String> = env::args().collect();
    if args.len() < 1 {
        eprintln!("usage: {} map.json", args[0]);
        std::process::exit(1);
    }

    let map_path: &Path = Path::new(&args[1]);
    let file_content = fs::read_to_string(map_path)?;

    let song_data: SongData = serde_json::from_str(&file_content)?;

    let scroll_speed: f32 = 1.5;
    let bpm: f32 = song_data.bpm;
    let mut notes_to_draw: Vec<Note> = song_data.notes;
    let mut current_judgement: Judgment = Judgment::None;

    let (mut rhl, rt) = raylib::init()
        .title(PROGRAM_NAME)
        .log_level(TraceLogLevel::LOG_NONE)
        .build();
    rhl.set_target_fps(60);
    notes_to_draw = notes_to_draw.into_iter().filter(|n| !n.empty).collect();
    let start_time: f32 = 2.0; // headstart
    let mut current_time: f32 = 0.0;
    let audio_device = audio::RaylibAudio::init_audio_device()?;
    let song = audio_device.new_music(&song_data.song)?;
    let mut song_playing = false;
    let mut score: i32 = 0;
    let mut timer = 0.;
    println!("Loaded notes: {}", notes_to_draw.len());
    let tap_sfx = audio_device.new_sound("taiko_ka.wav")?;

    let lane_x_positions: [(i32, KeyboardKey); 4] = [
        (
            rhl.get_screen_width() / 2 - 2 * LANE_WIDTH,
            KeyboardKey::KEY_D,
        ),
        (
            rhl.get_screen_width() / 2 - 1 * LANE_WIDTH,
            KeyboardKey::KEY_F,
        ),
        (
            rhl.get_screen_width() / 2 + 0 * LANE_WIDTH,
            KeyboardKey::KEY_J,
        ),
        (
            rhl.get_screen_width() / 2 + 1 * LANE_WIDTH,
            KeyboardKey::KEY_K,
        ),
    ];
    while !rhl.window_should_close() {
        let mut d = rhl.begin_drawing(&rt);
        timer += d.get_frame_time();

        if current_time > 0.0 {
            if !song_playing {
                song_playing = true;
                song.play_stream();
                song.seek_stream(song_data.offset);
            } else {
                song.update_stream();
                current_time = song.get_time_played();
            }
        } else {
            current_time = timer - start_time;
        }

        for (x_pos, _) in lane_x_positions {
            d.draw_rectangle(
                x_pos,
                0,
                LANE_WIDTH,
                d.get_screen_width(),
                Color::new(50, 25, 50, 255),
            );
            d.draw_rectangle(
                x_pos,
                0,
                2,
                d.get_screen_width(),
                Color::new(255, 255, 255, 64),
            );
        }

        for (lane, (x_pos, key_code)) in lane_x_positions.iter().enumerate() {
            if d.is_key_pressed(*key_code) {
                tap_sfx.play();
                let judgement = check_note_hit(&mut notes_to_draw, lane as u8+1, current_time);

                match judgement {
                    Judgment::Flawless => {
                        score += 100;
                    }
                    Judgment::Perfect => {
                        score += 50;
                    }
                    Judgment::Nice => {
                        score += 10;
                    }
                    _ => {}
                }

                current_judgement = judgement;

                d.draw_line(
                    *x_pos,
                    RECEPTOR_Y as i32,
                    x_pos + LANE_WIDTH,
                    RECEPTOR_Y as i32,
                    Color::WHITE,
                );
            } else {
                d.draw_line(
                    *x_pos,
                    RECEPTOR_Y as i32,
                    x_pos + LANE_WIDTH,
                    RECEPTOR_Y as i32,
                    Color::GRAY,
                );
            }
        }

        for note in notes_to_draw.iter_mut() {
            if note.state == Judgment::None && current_time > note.time + TIME_MISS {
                note.state = Judgment::Miss; // since we immediately set to miss, this check wont pass the next time it's made
                current_judgement = note.state;
                score = score.saturating_sub(50);
            }
        }

        d.draw_text(
            &Judgment::string(current_judgement),
            lane_x_positions[1].0,
            d.get_screen_width() / 2,
            58,
            Color::RED,
        );

        d.clear_background(Color::from_hex("1f0c42")?);

        for note in notes_to_draw.iter() {
            let time_diff = note.time - current_time;
            let effective_beats = time_diff as f32 * (bpm / 60.0);

            let scroll_factor = d.get_screen_height() as f32 / scroll_speed;
            let distance_from_receptor_y = effective_beats as f32 * scroll_factor;
            let note_y = RECEPTOR_Y - NOTE_HEIGHT - distance_from_receptor_y;

            if note_y > d.get_screen_height() as f32 {
                continue;
            }

            if note_y < RECEPTOR_Y - d.get_screen_height() as f32 {
                continue;
            }

            if note.state == Judgment::None || note.state == Judgment::Miss {
                let lane_index = note.lane as usize;
                let note_x = lane_x_positions[lane_index - 1].0;

                let color = if note.state == Judgment::Miss {
                    Color::RED
                } else {
                    Color::color_from_hsv(0., 0.6, 1.)
                };

                d.draw_rectangle(note_x, note_y as i32, LANE_WIDTH, NOTE_HEIGHT as i32, color);
            }
        }

        let fps = d.get_fps();
        draw_text_shadow(&mut d, PROGRAM_NAME.to_string(), 0);
        draw_text_shadow(&mut d, format!("{map_path:#?}"), 1);
        draw_text_shadow(&mut d, format!("Time: {:.2}", current_time), 2);
        draw_text_shadow(&mut d, format!("Score: {score}"), 3);
        draw_text_shadow(&mut d, format!("FPS: {fps}"), 4);
        d.draw_text(
            &Judgment::string(current_judgement),
            lane_x_positions[1].0,
            d.get_screen_width() / 2,
            58,
            Color::RED,
        );
    }
    Ok(())
}
