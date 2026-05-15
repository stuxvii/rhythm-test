use crate::{
    design,
    models::{Align, AppState, Screens},
    parser,
};
use raylib::prelude::*;

pub(crate) fn handle_menu_screen<'a>(d: &mut RaylibDrawHandle, audio_device: &'a RaylibAudio, state: &mut AppState, current_song: &mut Option<Music<'a>>) -> Result<bool, Box<dyn std::error::Error>> {
    let mut draw_label = |text: &str, y_offset: i32| {
        design::draw_text(d, text, Align::Middle, Align::Middle, 20, state.ui.fg, (0, y_offset), &state.ui);
    };

    if let Some(ref s) = state.song_state.song_data {
        draw_label(&s.name, -60);
        draw_label(&format!("Notes: {}", s.notes.len()), -40);
        draw_label(&s.metadata.name, -20);

        design::draw_text(d, "Hold shift to autoplay", Align::End, Align::Middle, 20, state.ui.fg, (0, 0), &state.ui);
        design::draw_text(d, "Press space to begin...", Align::Middle, Align::Middle, 30, state.ui.fg, (0, 0), &state.ui);

        if d.is_key_pressed(KeyboardKey::KEY_SPACE) {
            state.song_state.timer = 0.;
            state.song_state.song_timer = 0.;
            state.current_screen = Screens::Game;
            state.song_modifiers.autoplay = d.is_key_down(KeyboardKey::KEY_LEFT_SHIFT);
        }
    } else {
        if d.is_file_dropped() {
            if let Some(raw_path) = d.load_dropped_files().paths().get(0) {
                match parser::parse_song_data(&std::path::PathBuf::from(raw_path)) {
                    Ok(song_data) => {
                        *current_song = Some(audio_device.new_music(&song_data.metadata.song)?);
                        state.song_state.song_data = Some(song_data);
                    }
                    Err(e) => msgbox::create("Error loading map :<", e.to_string().as_str(), msgbox::IconType::Error)?,
                };
            }
        }

        design::draw_text(d, "q - quit", Align::End, Align::Start, 20, state.ui.fg, (10, -10), &state.ui);
        design::draw_text(d, "e - settings & info", Align::End, Align::Start, 20, state.ui.fg, (10, -30), &state.ui);
        design::draw_text(d, "[space] - maps", Align::End, Align::Start, 20, state.ui.fg, (10, -50), &state.ui);
        design::draw_text(d, "acidbox93's tabernacular", Align::Start, Align::End, 50, state.ui.fg, (-10, 10), &state.ui);
        design::draw_text(d, "offline quaver", Align::Start, Align::End, 50, state.ui.fg, (-10, 60), &state.ui);
        design::draw_text(d, "chart player", Align::Start, Align::End, 50, state.ui.fg, (-10, 110), &state.ui);

        if d.is_key_pressed(KeyboardKey::KEY_Q) {
            return Ok(true);
        }
        if d.is_key_pressed(KeyboardKey::KEY_SPACE) {
            state.current_screen = Screens::Songs;
        }
        if d.is_key_pressed(KeyboardKey::KEY_E) {
            state.current_screen = Screens::Settings;
        }
    }

    Ok(false)
}
