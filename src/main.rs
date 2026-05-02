use raylib::prelude::*;
mod design;
mod game;
mod judgment;
mod menu;
mod models;
mod parser;
mod results;
use crate::models::*;

fn main_loop() -> Result<(), Box<dyn std::error::Error>> {
    let (mut rhl, rt) = raylib::init().log_level(TraceLogLevel::LOG_NONE).resizable().height(480).width(640).msaa_4x().build();
    let mut state: AppState = AppState::init()?;
    let audio_device: RaylibAudio = audio::RaylibAudio::init_audio_device()?;

    rhl.set_window_min_size(640, 480);
    rhl.set_target_fps(state.game_config.max_fps);
    rhl.set_exit_key(None);

    for n in 0..4 {
        // now i know... this is ugly... but text handling is not joyous in this platform. :(
        let f = rhl.load_font_from_memory(&rt, ".ttf", include_bytes!("../lt.ttf"), 10 + (n * state.ui.text_scale), None)?;
        f.texture().set_texture_filter(&rt, TextureFilter::TEXTURE_FILTER_TRILINEAR);
        state.ui.fonts.push(f);
    }

    let mut current_song: Option<Music<'_>> = None;
    let mut current_tap: Option<Sound<'_>> = Some(audio_device.new_sound("./hit.wav")?);
    audio_device.set_master_volume(0.05);
    while !rhl.window_should_close() {
        let mut d = rhl.begin_drawing(&rt);
        state.viewport.h = d.get_screen_height();
        state.viewport.w = d.get_screen_width();
        d.clear_background(Color::BLANK);
        state.viewport.receptor_y = state.viewport.h - state.ui.note_height;
        if d.is_key_pressed(KeyboardKey::KEY_ESCAPE) {
            state.current_screen = Screens::Menu;
            state.song_state = PlayState::new();
            current_song = None;
        }
        match state.current_screen {
            Screens::Game => {
                if let Some(song_data) = &state.song_state.song_data {
                    state.viewport.lanes = (0..song_data.lanes)
                        .map(|i| {
                            let mut offset = (i - (song_data.lanes) / 2) * state.ui.lane_width;
                            offset += state.ui.lane_width / 2;
                            let key = state.keys.get(i as usize).unwrap_or(&KeyboardKey::KEY_ZERO);
                            (state.viewport.w / 2 + offset, *key)
                        })
                        .collect();

                    game::game_loop(d, &mut state, &current_tap, &current_song);
                }
            }
            Screens::Menu => {
                let result = menu::handle_menu_screen(&mut d, &audio_device, &mut state, &mut current_song);
                if let Ok(should_break) = result && should_break {
                    break;
                } else if let Err(err) = result {
                    return Err(err)
                }
            }
            Screens::Results => results::draw_results(d, &mut state, &current_song),
            Screens::Songs => {}
        }
    }

    std::fs::write("config.json", state.json().to_string())?;
    Ok(())
}

fn main() {
    if let Err(e) = main_loop() {
        msgbox::create("A bubonically fatal error just ocurred.", e.to_string().as_str(), msgbox::IconType::Error).unwrap()
    };
}
