use raylib::prelude::*;
use serde_json::json;
mod design;
mod game;
mod judgment;
mod models;
mod parser;
mod results;
use crate::{models::*, results::draw_results};

fn main_loop() -> Result<(), Box<dyn std::error::Error>> {
    let (mut rhl, rt) = raylib::init().log_level(TraceLogLevel::LOG_NONE).resizable().height(480).width(640).msaa_4x().build();
    let mut state: AppState = AppState::init();
    rhl.set_window_min_size(640, 480);
    rhl.set_target_fps(state.game_config.max_fps);
    rhl.set_exit_key(None);
    let audio_device: RaylibAudio = audio::RaylibAudio::init_audio_device()?;
    let mut song: Option<Music> = None;

    for n in 0..4 {
        // now i know... this is ugly... but text handling is not joyous in this platform. :(
        let f = rhl.load_font_from_memory(&rt, ".ttf", include_bytes!("../lt.ttf"), 10 + (n * state.ui.text_scale), None)?;
        f.texture().set_texture_filter(&rt, TextureFilter::TEXTURE_FILTER_TRILINEAR);
        state.ui.fonts.push(f);
    }

    let tap_wave = audio_device.new_wave_from_memory(".wav", &include_bytes!("../hit.wav").to_vec())?;
    let tap_sfx = audio_device.new_sound_from_wave(&tap_wave)?;
    audio_device.set_master_volume(0.05);
    while !rhl.window_should_close() {
        let mut d = rhl.begin_drawing(&rt);
        state.viewport.w = d.get_screen_width();
        state.viewport.h = d.get_screen_height();
        d.clear_background(state.ui.bg);
        state.viewport.receptor_y = state.viewport.h - state.ui.note_height;
        if d.is_key_pressed(KeyboardKey::KEY_ESCAPE) {
            state.current_screen = Screens::Menu;
            state.song_state = SongState::new();
            song = None;
        }
        match state.current_screen {
            Screens::Game => {
                if let Some(song_data) = &state.song_state.song_data {
                    let lane_x_positions: Vec<(i32, KeyboardKey)> = (0..song_data.lanes)
                        .map(|i| {
                            let mut offset = (i - (song_data.lanes) / 2) * state.ui.lane_width;
                            offset += state.ui.lane_width / 2;
                            let key = state.keys.get(i as usize).unwrap_or(&KeyboardKey::KEY_ZERO);
                            (state.viewport.w / 2 + offset, *key)
                        })
                        .collect();

                    state.viewport.lanes = lane_x_positions;

                    if let Some(song) = &mut song {
                        game::game_loop(d, &mut state, song, &tap_sfx);
                    }
                }
            }
            Screens::Menu => {
                let mut draw_label = |text: &str, y_offset: i32| {
                    design::draw_text(&mut d, text, Align::Middle, Align::Middle, 20, state.ui.fg, (0, y_offset), &state.ui);
                };

                if let Some(ref s) = state.song_state.song_data {
                    draw_label(&s.name, -60);
                    draw_label(&format!("Notes: {}", s.notes.len()), -40);
                    draw_label(&s.difficulty_name, -20);
                    design::draw_text(&mut d, "Hold shift to autoplay", Align::End, Align::Middle, 20, state.ui.fg, (0, 0), &state.ui);
                    design::draw_text(&mut d, "Press space to begin...", Align::Middle, Align::Middle, 30, state.ui.fg, (0, 0), &state.ui);

                    if d.is_key_pressed(KeyboardKey::KEY_SPACE) {
                        state.song_state.timer = 0.;
                        state.song_state.song_timer = 0.;
                        state.current_screen = Screens::Game;
                        state.song_modifiers.autoplay = d.is_key_down(KeyboardKey::KEY_LEFT_SHIFT);
                    }
                } else {
                    design::draw_text(&mut d, "q - quit", Align::End, Align::Start, 20, state.ui.fg, (10, -10), &state.ui);
                    design::draw_text(&mut d, "space - maps", Align::End, Align::Start, 20, state.ui.fg, (10, -30), &state.ui);
                    if d.is_file_dropped() {
                        if let Some(raw_path) = d.load_dropped_files().paths().get(0) {
                            match parser::parse_song_data(&std::path::PathBuf::from(raw_path)) {
                                Ok(song_data) => {
                                    let s = audio_device.new_music(&song_data.song)?;
                                    s.set_pitch(1.5);
                                    song = Some(s);
                                    state.song_state.song_data = Some(song_data);
                                }
                                Err(e) => msgbox::create("Error loading map :<", e.to_string().as_str(), msgbox::IconType::Error)?,
                            };
                        }
                    }
                }
            }
            Screens::Results => {
                if let Some(ref song) = song {
                    song.update_stream();
                }
                draw_results(d, &mut state);
            }
            Screens::Songs => {}
        }
    }

    let whatever = json!({
        "scroll_speed": state.game_config.scroll_speed,
        "visual_offset":state.game_config.visual_offset,
        "input_offset": state.game_config.input_offset,
        "max_fps":      state.game_config.max_fps,
        "lane_1_key":   state.game_config.lane_1_key,
        "lane_2_key":   state.game_config.lane_2_key,
        "lane_3_key":   state.game_config.lane_3_key,
        "lane_4_key":   state.game_config.lane_4_key,
        "songs_path":   state.game_config.songs_path,
    });

    std::fs::write("config.json", whatever.to_string())?;

    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    if let Err(e) = main_loop() {
        msgbox::create("A bubonically fatal error just ocurred.", e.to_string().as_str(), msgbox::IconType::Error)?
    };
    Ok(())
}
