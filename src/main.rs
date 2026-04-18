use raylib::prelude::*;
use serde_json::json;
use std::env;
mod game;
mod judgment;
mod models;
mod results;
use crate::{models::*, results::draw_results};

struct UIElements {
    pub fonts: Vec<Font>,
    pub lane_width: i32,
    pub note_height: i32,
    pub fg_color: Color,
    pub bg_color: Color,
}

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
        .log_level(TraceLogLevel::LOG_NONE)
        .resizable()
        .height(480)
        .width(640)
        .msaa_4x()
        .build();

    rhl.set_window_min_size(640, 480);
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

    let main_font: Font = rhl.load_font_ex(&rt, "daydream.otf", 60, None)?;
    main_font.texture().set_texture_filter(&rt, TextureFilter::TEXTURE_FILTER_TRILINEAR);

    let ui_state: UIElements = UIElements { 
        fonts: vec![main_font], 
        lane_width: 100, 
        note_height: 20, 
        fg_color: Color::WHITE,
        bg_color: Color::BLACK
    };

    let tap_sfx = audio_device.new_sound(&game_config.hitsound)?;
    while !rhl.window_should_close() {
        let mut d = rhl.begin_drawing(&rt);
        let sc_wdth_half = d.get_screen_width() / 2;
        screen_dimensions.w = d.get_screen_width();
        screen_dimensions.h = d.get_screen_height();
        d.clear_background(ui_state.bg_color);
        in_game_state.receptor_y = screen_dimensions.h - ui_state.note_height;
        match in_game_state.current_screen {
            Screens::Game => {
                let sg_clone = &in_game_state.song_data.clone();
                let num_lanes = sg_clone.clone().unwrap().lanes.clone();
                let lane_x_positions: Vec<(i32, KeyboardKey)> = (0..num_lanes)
                    .map(|i| {
                        let offset = (i as f32 - (num_lanes as f32 - 1.0) / 2.0) * ui_state.lane_width as f32;
                        let key = keys.get(i as usize).unwrap_or(&KeyboardKey::KEY_ZERO);
                        (sc_wdth_half + offset as i32, *key)
                    })
                    .collect();

                in_game_state.lanes = lane_x_positions;

                if d.is_key_pressed(KeyboardKey::KEY_F1) {
                    in_game_state = ProgramState::new(in_game_state.lanes, in_game_state.receptor_y);
                    song = None;
                }
                game_config.scroll_speed += if d.is_key_down(KeyboardKey::KEY_LEFT_SHIFT) {
                    d.get_mouse_wheel_move() / 10.
                } else {
                    d.get_mouse_wheel_move()
                };

                if let Some(song) = &mut song {
                    game::game_loop(d, screen_dimensions, &mut in_game_state, song, &tap_sfx, &game_config, &ui_state);
                }
            }
            Screens::Menu => {
                let mut draw_label = |text: &str, y_offset: i32| {
                    // ease the pain a little...
                    Align::draw_text(&mut d, text, Align::Middle, Align::Middle, 20, ui_state.fg_color, Some((0, y_offset)), false, &ui_state);
                };

                if let Some(ref s) = in_game_state.song_data {
                    draw_label(&s.name, -60);
                    draw_label(&format!("Notes: {}", s.notes.len()), -40);
                    draw_label(&s.difficulty_name, -20);

                    Align::draw_text(&mut d, "Hold shift to autoplay", Align::End, Align::Middle, 20, ui_state.fg_color, None, false, &ui_state);
                    Align::draw_text(&mut d, "Press space to begin...", Align::Middle, Align::Middle, 30, ui_state.fg_color, None, false, &ui_state);

                    if d.is_key_pressed(KeyboardKey::KEY_SPACE) {
                        in_game_state.current_timer = 0.;
                        in_game_state.current_song_timer = 0.;
                        in_game_state.current_screen = Screens::Game;
                        game_config.autoplay = d.is_key_down(KeyboardKey::KEY_LEFT_SHIFT);
                    }
                } else {
                    draw_label(&current_error, 20);

                    Align::draw_text(&mut d, "SUBYSMDSLAGC", Align::Start, Align::End, 30, ui_state.fg_color, Some((-20, 20)), false, &ui_state);
                    if d.is_file_dropped() {
                        let dropped_files = d.load_dropped_files();

                        if let Some(raw_path) = dropped_files.paths().get(0) {
                            match SongData::setup_map_and_get_song(raw_path.to_string(), &mut in_game_state) {
                                Ok(s) => song = Some(audio_device.new_music(s.to_str().unwrap())?),
                                Err(e) => current_error = e.to_string(),
                            };
                        }
                    }
                    Align::calculate_position(&mut d, Align::End, Align::End, Some((0, 20)));
                }
            }
            Screens::Results => {
                draw_results(d, &mut in_game_state, &game_config, &mut song, &ui_state);
            }
        }
    }

    let whatever = json!({
        "scroll_speed":  game_config.scroll_speed,
        "visual_offset": game_config.visual_offset,
        "input_offset":  game_config.input_offset,
        "max_fps": game_config.max_fps,
        "hitsound": game_config.hitsound,
        "lane_1_key": game_config.lane_1_key,
        "lane_2_key": game_config.lane_2_key,
        "lane_3_key": game_config.lane_3_key,
        "lane_4_key": game_config.lane_4_key,
        "lane_5_key": game_config.lane_5_key,
        "lane_6_key": game_config.lane_6_key,
        "lane_7_key": game_config.lane_7_key
    });

    let str = whatever.to_string();
    println!("{str}");
    std::fs::write("config.json", str).unwrap();
    
    Ok(())
}
