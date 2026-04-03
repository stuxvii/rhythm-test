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

    let lane_1_key = input::key_from_i32(game_config.lane_1_key).unwrap_or(KeyboardKey::KEY_D);
    let lane_2_key = input::key_from_i32(game_config.lane_2_key).unwrap_or(KeyboardKey::KEY_F);
    let lane_3_key = input::key_from_i32(game_config.lane_3_key).unwrap_or(KeyboardKey::KEY_J);
    let lane_4_key = input::key_from_i32(game_config.lane_4_key).unwrap_or(KeyboardKey::KEY_K);

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
    let mut in_game_state: ProgramState = ProgramState::new([(0, lane_1_key), (0, lane_2_key), (0, lane_3_key), (0, lane_4_key)], 0);

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
        let lane_x_positions: [(i32, KeyboardKey); 4] = [
            (sc_wdth_half - (2 * LANE_WIDTH), lane_1_key),
            (sc_wdth_half - (1 * LANE_WIDTH), lane_2_key),
            (sc_wdth_half + (0 * LANE_WIDTH), lane_3_key),
            (sc_wdth_half + (1 * LANE_WIDTH), lane_4_key),
        ];
        screen_dimensions.w = d.get_screen_width();
        screen_dimensions.h = d.get_screen_height();
        d.clear_background(Color::BLACK);
        in_game_state.lanes = lane_x_positions;
        in_game_state.receptor_y = screen_dimensions.h - NOTE_HEIGHT;
        if in_game_state.song_data.is_some() {
            if in_game_state.current_screen == Screens::Menu {
                Align::draw_text(&mut d, "Press space to begin...", Align::Middle, Align::Middle, 30, BG_COLOR, None, false);
                Align::draw_text(&mut d, "Hold shift to autoplay", Align::Middle, Align::Middle, 10, BG_COLOR, Some(Vector2::new(0., 20.)), false);
                if d.is_key_pressed(KeyboardKey::KEY_SPACE) {
                    in_game_state.current_screen = Screens::Game;
                    game_config.autoplay = d.is_key_down(KeyboardKey::KEY_LEFT_SHIFT);
                }
            }
        } else {
            Align::draw_text(&mut d, "Please drop in a chart file!", Align::Middle, Align::Middle, 30, BG_COLOR, None, false);
            Align::draw_text(&mut d, &current_error, Align::Middle, Align::Middle, 10, BG_COLOR, Some(Vector2::new(0., 20.)), false);
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
