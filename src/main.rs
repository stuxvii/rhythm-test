pub use models::*;
pub use raylib::prelude::*;
pub mod design;
pub mod game;
pub mod judgment;
pub mod menu;
pub mod models;
pub mod parser;
pub mod results;
pub mod songs;

fn main_loop(rhl: &mut RaylibHandle, rt: &RaylibThread) -> Result<(), Box<dyn std::error::Error>> {
    let mut state: AppState = AppState::init()?;

    state.interaction.tried_loading_charts = !state.config.auto_fetch;

    rhl.set_window_min_size(800, 600);
    rhl.set_target_fps(state.config.max_fps);
    rhl.set_exit_key(None);

    if let Ok(data) = std::fs::read("font.ttf") {
        for n in 0..5 {
            // now i know... this is ugly... but text handling is not joyous in this platform. :(
            let f = rhl.load_font_from_memory(&rt, ".ttf", &data, 10 + (n * state.ui.text_scale), None)?;
            f.texture().set_texture_filter(&rt, TextureFilter::TEXTURE_FILTER_TRILINEAR);
            state.ui.fonts.push(f);
        }
    }
    let audio_device: RaylibAudio = audio::RaylibAudio::init_audio_device()?;

    let mut soundscape = Soundscape {
        current_song: None,
        current_tap: Some(audio_device.new_sound("./hit.wav")?),
        option_sfx: audio_device.new_sound("./option.wav")?,
        cancel_sfx: audio_device.new_sound("./cancel.wav")?,
    };

    let fs_starry_field = include_str!("star.fs");
    let mut shader = rhl.load_shader_from_memory(&rt, None, Some(fs_starry_field));
    let time_loc = shader.get_shader_location("uTime");
    let mut t: f32 = 0.0;

    while !rhl.window_should_close() {
        let mut d = rhl.begin_drawing(&rt);
        state.viewport.h = d.get_screen_height();
        state.viewport.w = d.get_screen_width();
        d.clear_background(Color::BLANK);
        state.viewport.receptor_y = state.viewport.h - state.ui.note_height - 4;
        if d.is_key_pressed(KeyboardKey::KEY_ESCAPE) {
            soundscape.cancel_sfx.play();
            state.current_screen = Screens::StartMenu;
            state.song_state = PlayState::new();
            soundscape.current_song = None;
            state.interaction.chosen_song = None;
            state.interaction.select_offset = 0;
            state.interaction.chosen_difficulty = 0;
            d.set_target_fps(state.config.max_fps);
        }
        if state.config.starry_field {
            if let Some(a) = &state.song_state.song_data {
                t += d.get_frame_time() * a.get_bpm(state.song_state.song_timer) / 100.;
            } else {
                t += d.get_frame_time();
            }
            shader.set_shader_value(time_loc, t);
            let mut sh_mode = d.begin_shader_mode(&mut shader);
            sh_mode.draw_rectangle(0, 0, state.viewport.w, state.viewport.h, Color::WHITE);
        }
        match state.current_screen {
            Screens::Game => {
                if let Some(song_data) = &state.song_state.song_data {
                    state.viewport.lanes = (0..song_data.metadata.lanes)
                        .map(|i| {
                            let mut offset = (i - (song_data.metadata.lanes) / 2) * state.ui.lane_width;
                            offset += state.ui.lane_width / 2;
                            let key = state.keys.get(i as usize).unwrap_or(&KeyboardKey::KEY_ZERO);
                            (state.viewport.w / 2 + offset, *key)
                        })
                        .collect();

                    game::game_loop(d, &mut state, &soundscape)?;
                }
            }
            Screens::StartMenu => {
                let result = menu::handle_menu_screen(&mut d, &audio_device, &mut state, &mut soundscape);
                if let Ok(should_break) = result
                    && should_break
                {
                    break;
                } else if let Err(err) = result {
                    return Err(err);
                }
            }
            Screens::Results => results::draw_results(d, &mut state, &soundscape.current_song),
            Screens::Songs => songs::draw_songs(d, &mut state, &mut soundscape, &audio_device, &rt)?,
            Screens::Settings => menu::draw_settings(d, &mut state, &soundscape.option_sfx),
        }
    }

    std::fs::write("config.json", state.json().to_string())?;
    Ok(())
}

fn main() {
    let (mut rhl, rt) = raylib::init()
        .log_level(TraceLogLevel::LOG_ALL)
        .resizable()
        .height(600)
        .width(800)
        .msaa_4x()
        .build();

    let mut error: Option<String> = None;

    if let Err(e) = main_loop(&mut rhl, &rt) {
        error = Some(e.to_string());
    };

    if let Some(e) = error {
        while !rhl.window_should_close() {
            let mut d = rhl.begin_drawing(&rt);
            d.clear_background(Color::BLACK);
            d.draw_text("An unrecoverable error has ocurred.", 10, 10, 20, Color::WHITE);
            d.draw_text(
                "If possible, reopen the game in a terminal and\ntrigger this error again, as to report and fix it.",
                10,
                30,
                20,
                Color::WHITE,
            );
            d.draw_text(e.as_str(), 10, 90, 20, Color::WHITE);
        }
    }
}
