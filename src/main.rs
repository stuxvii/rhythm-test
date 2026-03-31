use raylib::prelude::*;

use std::env;
use std::fs;
use std::path::Path;
use std::path::PathBuf;

mod judgement;
mod models;
use crate::judgement::Judgment;
use crate::models::*;

const PROGRAM_NAME: &str = "Rhythm";
const LANE_WIDTH: i32 = 100;
const NOTE_HEIGHT: i32 = 20;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", PROGRAM_NAME);
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("usage: {} map.json", args[0]);
        std::process::exit(1);
    }

    let map_path: &Path = Path::new(&args[1]);
    let map_dir = map_path.parent().unwrap_or(Path::new("."));
    let file_content = fs::read_to_string(map_path)?;

    let song_data: SongData = serde_json::from_str(&file_content)?;
    let game_config: GameConfig = GameConfig::load();

    let scroll_speed: f32 = game_config.scroll_speed;
    let bpm: f32 = song_data.bpm;
    let mut notes_to_draw: Vec<Note> = song_data.notes.clone();
    let mut cur_judge: Judgment = Judgment::None;

    let (mut rhl, rt) = raylib::init()
        .title(PROGRAM_NAME)
        .log_level(TraceLogLevel::LOG_NONE)
        .resizable()
        .height(640)
        .width(640)
        .msaa_4x()
        .build();
    rhl.set_window_min_size(640, 640);
    rhl.set_target_fps(game_config.max_fps);
    notes_to_draw = notes_to_draw.into_iter().filter(|n| !n.empty).collect();
    let start_time: f32 = 2.0; // headstart
    let mut current_song_time: f32 = 0.0;
    let mut current_time: f32 = 0.0;
    let audio_device = audio::RaylibAudio::init_audio_device()?;
    let song_path: PathBuf = map_dir.join(&song_data.song);
    let mut song = audio_device.new_music(song_path.to_str().unwrap())?;
    let mut song_playing = false;
    let tap_sfx = audio_device.new_sound(&game_config.hitsound)?;
    let mut combo = 0;
    while !rhl.window_should_close() {
        let mut d = rhl.begin_drawing(&rt);
        let screen_height = d.get_screen_height();
        let receptor_y = screen_height - NOTE_HEIGHT;
        let sc_wdth_half = d.get_screen_width() / 2;
        let lane_x_positions: [(i32, KeyboardKey); 4] = [
            (sc_wdth_half - (2 * LANE_WIDTH), KeyboardKey::KEY_A),
            (sc_wdth_half - (1 * LANE_WIDTH), KeyboardKey::KEY_S),
            (sc_wdth_half + (0 * LANE_WIDTH), KeyboardKey::KEY_K),
            (sc_wdth_half + (1 * LANE_WIDTH), KeyboardKey::KEY_L),
        ];

        // PROGRESS THE SONG AND MANAGE IT
        if current_song_time > 0.0 {
            if !song_playing {
                song_playing = true;
                song.play_stream();
                song.looping = false;
                song.seek_stream(song_data.offset);
            } else {
                if game_config.quit_after_song_end && current_song_time > song.get_time_length() + 5. {
                    break;
                } else if current_song_time < song.get_time_length() {
                    song.update_stream();
                    current_song_time = song.get_time_played();
                } else {
                    current_song_time += d.get_frame_time();
                }
            }
        } else {
            current_song_time = current_time + (song_data.offset - start_time);
        }

        current_time += d.get_frame_time();

        for (x_pos, _) in lane_x_positions {
            d.draw_rectangle(x_pos, 0, LANE_WIDTH, screen_height, Color::new(50, 25, 50, 255));
            d.draw_rectangle(x_pos, 0, 2, screen_height, Color::new(255, 255, 255, 64));
        }

        // HERE WE DO CHECKING FOR KEY HITS AND DRAWING THE FIELD ZONE DIFFERENTLY
        for (lane, (x_pos, key_code)) in lane_x_positions.iter().enumerate() {
            let lane_start_pos = Vector2::new(*x_pos as f32, receptor_y as f32);
            let lane_end_pos = Vector2::new(*x_pos as f32 + LANE_WIDTH as f32, receptor_y as f32);
            if d.is_key_pressed(*key_code) {
                tap_sfx.play();
                cur_judge = Note::check_note_hit(&mut notes_to_draw, lane + 1, current_song_time);
                if cur_judge == Judgment::Ehhh {
                    combo = 0;
                } else if cur_judge != Judgment::None {
                    println!("{}", cur_judge);
                    combo += 1;
                }

                // the thick: 5. here leaves a small gap where the hitzone is visible, kinda sucks on really high dpi screens.
                d.draw_line_ex(lane_start_pos, lane_end_pos, 10., Color::WHITE);
            } else if d.is_key_down(*key_code) {
                d.draw_line_ex(lane_start_pos, lane_end_pos, 10., Color::LIGHTGRAY);
            } else {
                d.draw_line_ex(lane_start_pos, lane_end_pos, 10., Color::GRAY);
            }
        }

        if game_config.autoplay {
            for note in notes_to_draw.iter_mut() {
                if current_song_time > note.time && note.state == Judgment::None {
                    note.state = Judgment::Perfect;
                    cur_judge = note.state;
                    tap_sfx.play();
                    combo += 1;

                    let lane_start_pos = Vector2::new(lane_x_positions[note.lane - 1].0 as f32, receptor_y as f32);
                    let lane_end_pos = Vector2::new(lane_x_positions[note.lane - 1].0 as f32 + LANE_WIDTH as f32, receptor_y as f32);
                    d.draw_line_ex(lane_start_pos, lane_end_pos, 2., Color::WHITE);
                }
            }
        } else {
            for note in notes_to_draw.iter_mut() {
                if note.is_missed(current_song_time) {
                    note.state = Judgment::Miss; // since we immediately set to miss, this check wont pass the next time it's made
                    note.accuracy = 10.;
                    cur_judge = note.state;
                    combo = 0;
                }
            }
        }

        // ALL THE PURELY VISUAL STUFF!!
        d.clear_background(Color::DARKPURPLE);
        for note in notes_to_draw.iter() {
            let time_diff = note.time - current_song_time;
            let effective_beats = time_diff as f32 * (bpm / 60.0);

            let scroll_factor = (screen_height as f32 * scroll_speed) / 50.;
            let distance_from_receptor_y = effective_beats * scroll_factor;
            let note_y = receptor_y - NOTE_HEIGHT - distance_from_receptor_y as i32;

            if note_y > screen_height {
                continue;
            }

            if note_y < receptor_y - screen_height {
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
        let bg_color = Color::WHITE;
        let fg_color = Color::BLACK;
        Align::draw_text(&mut d, &format!("FPS: {fps}"), Align::Start, Align::Start, 20, bg_color, None);

        if let Some(last_note) = notes_to_draw.last() {
            let complete_ratio = current_song_time / last_note.time;
            d.draw_rectangle(0, screen_height - NOTE_HEIGHT, d.get_screen_width(), NOTE_HEIGHT, Color::GRAY);
            d.draw_rectangle(
                0,
                screen_height - NOTE_HEIGHT,
                (complete_ratio * d.get_screen_width() as f32) as i32,
                NOTE_HEIGHT,
                bg_color,
            );

            let offset = Some(Vector2::new(0., 1.));

            let minutes_cur_time = (current_song_time as i32 / 60) % 60;
            let seconds_cur_time = current_song_time as i32 % 60;
            let text_cur_time = format!("{:0>2}:{:0>2}", minutes_cur_time, seconds_cur_time);

            let minutes_rem_time = ((last_note.time - current_song_time) as i32 / 60) % 60;
            let seconds_rem_time = (last_note.time - current_song_time) as i32 % 60;
            let text_rem_time = format!("{:0>2}:{:0>2}", minutes_rem_time, seconds_rem_time);

            Align::draw_text(&mut d, &song_data.name, Align::End, Align::Middle, NOTE_HEIGHT, fg_color, None);
            Align::draw_text(&mut d, &text_rem_time, Align::End, Align::End, NOTE_HEIGHT, fg_color, offset);
            Align::draw_text(&mut d, &text_cur_time, Align::End, Align::Start, NOTE_HEIGHT, fg_color, offset);
        }

        let accuracy_txt = if game_config.autoplay {
            format!("AUTOPLAY")
        } else {
            format!("{:.2}%", Note::accuracy(&notes_to_draw).clamp(0., 100.))
        };
        let misses: Vec<&Note> = notes_to_draw.iter().filter(|n| n.state == Judgment::Miss).collect();
        let misses_txt = format!("Misses: {}", misses.len());
        let combo_txt = format!("{combo}");
        let judg_txt = format!("{}", cur_judge);

        Align::draw_text(&mut d, &accuracy_txt, Align::Start, Align::Middle, 20, bg_color, None);
        Align::draw_text(&mut d, &misses_txt, Align::Start, Align::End, 20, bg_color, None);
        Align::draw_text(&mut d, &combo_txt, Align::Middle, Align::Middle, 20, bg_color, Some(Vector2::new(0., 20.)));
        Align::draw_text(&mut d, &judg_txt, Align::Middle, Align::Middle, 30, bg_color, None);
    }
    Ok(())
}
