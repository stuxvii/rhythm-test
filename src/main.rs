use macroquad::miniquad::conf::Platform;
use macroquad::ui::widgets::Slider;
use macroquad::ui::{
    hash, root_ui,
    widgets::{self},
};
use macroquad::{audio, prelude::*};
use std::env;
use std::fs;
use std::path::Path;

const PROGRAM_NAME: &str = "Rhythm";
const LANE_WIDTH: f32 = 100.0;
const LANE_HEIGHT: f32 = 700.0;
const NOTE_HEIGHT: f32 = 10.0;
const RECEPTOR_Y: f32 = 640.0;
const TIME_ORGASMICAL: f64 = 0.025;
const TIME_PERFECT: f64 = 0.050;
const TIME_NICE: f64 = 0.100;

#[derive(Debug, Clone, Copy, PartialEq)]
enum Judgment {
    Orgasmical,
    Perfect,
    Nice,
    Miss,
    None,
}

impl Judgment {
    fn string(j: Judgment) -> String {
        match j {
            Judgment::Orgasmical => {
                String::from("ORGASMICAL!!!")
            },
            Judgment::Perfect => {
                String::from("PERFECT!!!")
            },
            Judgment::Nice => {
                String::from("NICE ASF!!!")
            },
            Judgment::Miss => {
                String::from("MISS! FUCKER!")
            },
            Judgment::None => {
                String::from("")
            },
        }
    }
}
struct Note {
    lane: u8,
    bpm: f32,
    expected_time: f64,
    state: Judgment,
    empty: bool,
}

impl Note {
    fn new(lane: u8, expected_time: f64, bpm: f32) -> Note {
        return Note {
            lane: lane,
            bpm: bpm,
            expected_time: expected_time,
            state: Judgment::None,
            empty: false,
        };
    }
    fn empty() -> Note {
        return Note {
            lane: 0,
            bpm: 0.,
            empty: true,
            expected_time: 0.,
            state: Judgment::None,
        };
    }
}

fn window_conf() -> Conf {
    Conf {
        window_title: PROGRAM_NAME.to_owned(),
        window_width: 1024,
        window_height: 700,
        window_resizable: false,
        platform: Platform {
            swap_interval: Some(1),
            ..Default::default()
        },
        ..Default::default()
    }
}

fn get_judgment(time_diff: f64) -> Judgment {
    if time_diff <= TIME_ORGASMICAL {
        return Judgment::Orgasmical;
    } else if time_diff <= TIME_PERFECT {
        return Judgment::Perfect;
    } else if time_diff <= TIME_NICE {
        return Judgment::Nice;
    } else {
        return Judgment::Miss;
    }
}

fn check_note_hit(notes: &mut [Note], lane: u8, current_time: f64) -> Option<Judgment> {
    if let Some(note) = notes.iter_mut().find(|n| {
        n.lane == lane
            && n.state == Judgment::None
            && (n.expected_time - current_time).abs() <= TIME_NICE
    }) {
        let time_diff = (note.expected_time - current_time).abs();
        let judgment = get_judgment(time_diff);
        note.state = judgment;
        return Some(judgment);
    }
    return None;
}

fn draw_text_shadow(text: String, height: f32) {
    let new_height = height * 16.0;
    draw_text(&text, 1.0, new_height, 16.0, BLACK);
    let new_height = new_height + 1.0;
    draw_text(&text, 0.0, new_height, 16.0, WHITE);
}

#[macroquad::main(window_conf)]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", PROGRAM_NAME);
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("usage: {} map.json song.[mp3|wav|ogg]", args[0]);
        std::process::exit(1);
    }

    let map_path: &Path = Path::new(&args[1]);
    let file_content = fs::read_to_string(map_path)?;
    let lines = file_content.lines();

    let mut reading_notes_from_file = false;

    let mut scroll_speed: f32 = 8.0;
    let mut bpm: f32 = 20.0;
    let mut notes_to_draw: Vec<Note> = vec![];
    let mut current_beat_time: f64 = 0.0;
    let mut beat_duration: f64 = 60.0 / bpm as f64;
    let mut current_judgement: Judgment = Judgment::None;

    // map parsin logic
    for line in lines {
        if line.starts_with("bpm") {
            // bpm xxx (e.g bpm 120)
            let split: Vec<&str> = line.split(" ").collect();
            if let Some(bpm_str) = split.get(1) {
                bpm = bpm_str.parse::<f32>().unwrap();
                beat_duration = 60.0 / bpm as f64;
            }
            continue;
        }

        if line.starts_with("start") {
            reading_notes_from_file = true;
            continue;
        }

        if reading_notes_from_file {
            let mut current_line_bpm = bpm;
            current_beat_time += beat_duration;

            // dfjk xxx (e.g dfjk 120) (bpm change mid song)
            let split: Vec<&str> = line.split(" ").collect();
            if let Some(bpm_str) = split.get(1) {
                if let Ok(new_bpm) = bpm_str.parse::<f32>() {
                    current_line_bpm = new_bpm;
                }
            }

            if current_line_bpm != bpm {
                bpm = current_line_bpm;
                beat_duration = 60.0 / bpm as f64;
            }

            let note_chars: std::str::Chars<'_> = line.chars();
            for note in note_chars {
                let new_note: Note = match note {
                    'd' => Note::new(0, current_beat_time, current_line_bpm),
                    'f' => Note::new(1, current_beat_time, current_line_bpm),
                    'j' => Note::new(2, current_beat_time, current_line_bpm),
                    'k' => Note::new(3, current_beat_time, current_line_bpm),
                    _ => Note::empty(),
                };
                if !new_note.empty {
                    notes_to_draw.push(new_note);
                }
            }
        }
    }

    notes_to_draw = notes_to_draw.into_iter().filter(|n| !n.empty).collect();
    let mut start_time: f64 = get_time() + 2.0; // headstart
    let mut current_time: f64 = 0.0;
    let song = audio::load_sound(&args[2]).await?;
    let mut song_playing = false;
    let mut game_playing = false;
    let mut score: i32 = 0;

    let tap_sfx = audio::load_sound("tap.wav").await?;
    loop {
        widgets::Window::new(hash!(), vec2(640., 160.), vec2(300., 300.))
            .label("Settings")
            .ui(&mut *root_ui(), |ui| {
                let range = 1.0..64.0;
                Slider::new(hash!(), range.clone())
                    .label("Notes On Screen")
                    .ui(ui, &mut scroll_speed);
                if ui.button(None, "Start") {
                    if !game_playing {
                        start_time = get_time() + 2.0;
                        game_playing = true;
                    }
                }
            });

        if game_playing {
            current_time = get_time() - start_time;

            // Miss Logic
            for note in notes_to_draw.iter_mut() {
                if note.state == Judgment::None && current_time > note.expected_time + TIME_NICE {
                    note.state = Judgment::Miss;
                    current_judgement = note.state;
                    score = score.saturating_sub(50);
                }
            }

            if current_time > 0.0 && !song_playing {
                song_playing = true;
                audio::play_sound_once(&song);
            }
        }

        let lane_x_positions: [(f32, KeyCode); 4] = [
            (screen_height() / 2.0 - 2.0 * LANE_WIDTH, KeyCode::D),
            (screen_height() / 2.0 - 1.0 * LANE_WIDTH, KeyCode::F),
            (screen_height() / 2.0 + 0.0 * LANE_WIDTH, KeyCode::J),
            (screen_height() / 2.0 + 1.0 * LANE_WIDTH, KeyCode::K),
        ];

        clear_background(Color::from_hex(0x1f0c42));

        for (x_pos, _) in lane_x_positions {
            draw_rectangle(
                x_pos,
                0.,
                LANE_WIDTH,
                LANE_HEIGHT,
                Color::from_rgba(50, 25, 50, 255),
            );
            draw_rectangle(
                x_pos,
                0.,
                2.0,
                LANE_HEIGHT,
                Color::from_rgba(255, 255, 255, 64),
            );
        }

        for (lane, (x_pos, key_code)) in lane_x_positions.iter().enumerate() {
            if is_key_pressed(*key_code) {
                audio::play_sound_once(&tap_sfx);
                let judgement = match check_note_hit(&mut notes_to_draw, lane as u8, current_time) {
                    Some(j) => {j},
                    None => {Judgment::None},
                };

                match judgement {
                    Judgment::Orgasmical => {
                        score += 100;
                    }
                    Judgment::Perfect => {
                        score += 50;
                    }
                    Judgment::Nice => {
                        score += 10;
                    }
                    // Miss is already implemented way above
                    _ => {
                        // NO PENALTY. I am a ghost tapping die hard. Fight me.
                    }
                }

                current_judgement = judgement;

                draw_line(
                    *x_pos,
                    RECEPTOR_Y,
                    x_pos + LANE_WIDTH,
                    RECEPTOR_Y,
                    10.0,
                    WHITE,
                );
            } else {
                draw_line(
                    *x_pos,
                    RECEPTOR_Y,
                    x_pos + LANE_WIDTH,
                    RECEPTOR_Y,
                    10.0,
                    GRAY,
                );
            }
        }

        for note in notes_to_draw.iter() {
            let time_diff = note.expected_time - current_time;
            let effective_beats = time_diff * (note.bpm as f64 / 60.0);

            let scroll_factor = LANE_HEIGHT / scroll_speed;
            let distance_from_receptor_y = effective_beats as f32 * scroll_factor;
            let note_y = RECEPTOR_Y - NOTE_HEIGHT - distance_from_receptor_y;

            if note_y > RECEPTOR_Y + NOTE_HEIGHT {
                continue;
            }

            if note_y < RECEPTOR_Y - LANE_HEIGHT {
                continue;
            }

            if note.state == Judgment::None || note.state == Judgment::Miss { 
                 let lane_index = note.lane as usize;
                 let note_x = lane_x_positions[lane_index].0;
                 
                 let color = if note.state == Judgment::Miss {
                     RED
                 } else {
                     Color::new(0., 0.6, 1., 1.)
                 };

                 draw_rectangle(
                    note_x,
                    note_y,
                    LANE_WIDTH,
                    NOTE_HEIGHT,
                    color,
                );
            }
        }

        draw_text_shadow(PROGRAM_NAME.to_string(), 0.5);
        draw_text_shadow(format!("{:#?}", map_path), 1.5);
        draw_text_shadow(format!("Time: {:.2}", current_time), 2.5);
        draw_text_shadow(format!("Score: {}", score), 3.5);
        draw_text_shadow(format!("FPS: {}", macroquad::time::get_fps()), 4.5);
        draw_text(&Judgment::string(current_judgement), lane_x_positions[1].0, LANE_HEIGHT / 2., 58., RED);
        next_frame().await
    }
}