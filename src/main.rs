use raylib::prelude::*;
use std::env;
mod game;
mod judgment;
mod models;
mod results;
use crate::{models::*, results::draw_results};

const PROGRAM_NAME: &str = "Rhythm";
const LANE_WIDTH: i32 = 100;
const NOTE_HEIGHT: i32 = 20;
const BG_COLOR: Color = Color::WHITE;
const FG_COLOR: Color = Color::BLACK;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();

    let mut game_config: GameConfig = GameConfig::load();

    let keys: Vec<KeyboardKey> = vec![
        input::key_from_i32(game_config.lane_1_key).unwrap_or(KeyboardKey::KEY_A),
        input::key_from_i32(game_config.lane_2_key).unwrap_or(KeyboardKey::KEY_S),
        input::key_from_i32(game_config.lane_3_key).unwrap_or(KeyboardKey::KEY_D),
        input::key_from_i32(game_config.lane_4_key).unwrap_or(KeyboardKey::KEY_SPACE),
        input::key_from_i32(game_config.lane_5_key).unwrap_or(KeyboardKey::KEY_J),
        input::key_from_i32(game_config.lane_6_key).unwrap_or(KeyboardKey::KEY_K),
        input::key_from_i32(game_config.lane_7_key).unwrap_or(KeyboardKey::KEY_L),
    ];

    let mut current_error: String = String::new();

    let (mut rhl, rt) = raylib::init()
        .title(PROGRAM_NAME)
        .log_level(TraceLogLevel::LOG_NONE)
        .resizable()
        .height(640)
        .width(640)
        .msaa_4x()
        .vsync()
        .build();

    rhl.set_window_min_size(640, 640);
    rhl.set_target_fps(game_config.max_fps);

    let mut screen_dimensions: ScreenDimension = ScreenDimension::new(0, 0);
    let mut in_game_state: ProgramState = ProgramState::new(vec![], 0);

    let audio_device: RaylibAudio = audio::RaylibAudio::init_audio_device()?;
    let mut song: Option<Music> = None;

    if args.len() == 2 {
        match SongData::setup_map_and_get_song(args[1].clone(), &mut in_game_state) {
            Ok(s) => song = Some(audio_device.new_music(s.to_str().unwrap())?),
            Err(e) => current_error = e.to_string(),
        };
    }

    let tap_sfx = audio_device.new_sound(&game_config.hitsound)?;
    while !rhl.window_should_close() {
        let mut d = rhl.begin_drawing(&rt);
        let sc_wdth_half = d.get_screen_width() / 2;
        screen_dimensions.w = d.get_screen_width();
        screen_dimensions.h = d.get_screen_height();
        d.clear_background(Color::BLACK);
        in_game_state.receptor_y = screen_dimensions.h - NOTE_HEIGHT;
        if let Some(ref s) = in_game_state.song_data {
            if in_game_state.current_screen == Screens::Menu {
                Align::draw_text(&mut d, &s.name, Align::Middle, Align::Middle, 10, BG_COLOR, Some(Vector2::new(0., -40.)), false);
                Align::draw_text(&mut d, "Press space to begin...", Align::Middle, Align::Middle, 30, BG_COLOR, None, false);
                Align::draw_text(
                    &mut d,
                    &s.difficulty_name,
                    Align::Middle,
                    Align::Middle,
                    10,
                    BG_COLOR,
                    Some(Vector2::new(0., -20.)),
                    false,
                );
                Align::draw_text(
                    &mut d,
                    &format!("Notes: {}", s.notes.len()),
                    Align::Middle,
                    Align::Middle,
                    10,
                    BG_COLOR,
                    Some(Vector2::new(0., -30.)),
                    false,
                );
                Align::draw_text(&mut d, "Hold shift to autoplay", Align::End, Align::Middle, 10, BG_COLOR, None, false);
                if d.is_key_pressed(KeyboardKey::KEY_SPACE) {
                    in_game_state.current_screen = Screens::Game;
                    game_config.autoplay = d.is_key_down(KeyboardKey::KEY_LEFT_SHIFT);
                }
            }
        } else {
            Align::draw_text(&mut d, "Please drop in a chart file!", Align::Middle, Align::Middle, 30, BG_COLOR, None, false);
            Align::draw_text(
                &mut d,
                &current_error,
                Align::Middle,
                Align::Middle,
                10,
                BG_COLOR,
                Some(Vector2::new(0., 20.)),
                false,
            );
            if d.is_file_dropped() {
                let dropped_files = d.load_dropped_files();

                if let Some(raw_path) = dropped_files.paths().get(0) {
                    match SongData::setup_map_and_get_song(raw_path.to_string(), &mut in_game_state) {
                        Ok(s) => song = Some(audio_device.new_music(s.to_str().unwrap())?),
                        Err(e) => current_error = e.to_string(),
                    };
                }
            }
        }
        match in_game_state.current_screen {
            Screens::Game => {
                let sg_clone = &in_game_state.song_data.clone();
                let num_lanes = sg_clone.clone().unwrap().lanes.clone();
                let lane_x_positions: Vec<(i32, KeyboardKey)> = (0..num_lanes)
                    .map(|i| {
                        let offset = (i as f32 - (num_lanes as f32 - 1.0) / 2.0) * LANE_WIDTH as f32;
                        let key = keys.get(i as usize).unwrap_or(&KeyboardKey::KEY_ZERO);
                        (sc_wdth_half + offset as i32, *key)
                    })
                    .collect();

                in_game_state.lanes = lane_x_positions;

                if d.is_key_pressed(KeyboardKey::KEY_F1) {
                    in_game_state = ProgramState::new(in_game_state.lanes, in_game_state.receptor_y);
                    song = None;
                }
                if let Some(song) = &mut song {
                    game::game_loop(d, screen_dimensions, &mut in_game_state, song, &tap_sfx, &game_config);
                }
            }
            Screens::Menu => {}
            Screens::Results => {
                draw_results(d, &mut in_game_state, &game_config, &mut song);
            }
        }
    }
    Ok(())
}
