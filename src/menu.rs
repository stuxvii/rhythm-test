use crate::{design, models::*, parser};
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
        design::draw_text(d, "rhythm game", Align::Start, Align::End, 50, state.ui.fg, (-10, 60), &state.ui);
        design::draw_text(d, "chart player", Align::Start, Align::End, 50, state.ui.fg, (-10, 110), &state.ui);

        let nav_texts = vec![
            "Navigation keybinds",
            "[left]/[down]/[up]/[right] - D/F/J/K",
            "[confirm] - Z",
            "[back] - X",
            "",
            "Global keybinds",
            "[esc] - main menu",
        ];

        for (i, txt) in nav_texts.iter().rev().enumerate() {
            design::draw_text(d, txt, Align::End, Align::End, 20, state.ui.fg, (-10, (-20 * i as i32) - 5), &state.ui);
        }

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

pub fn draw_settings(mut d: RaylibDrawHandle<'_>, state: &mut AppState, option_sfx: &Sound<'_>) {
    if let Some(item) = ConfigItem::items().get(state.interaction.select_offset) {
        let nav_down = d.is_key_pressed_repeat(state.config.keybinds.left) || d.is_key_pressed(state.config.keybinds.left);
        let nav_up = d.is_key_pressed_repeat(state.config.keybinds.right) || d.is_key_pressed(state.config.keybinds.right);
        let mut delta = if nav_down {
            -1
        } else if nav_up {
            1
        } else {
            0
        };

        if delta != 0 {
            option_sfx.play();
        }

        if d.is_key_down(KeyboardKey::KEY_LEFT_SHIFT) {
            delta *= 10;
        }
        item.adjust(delta, state);
    }

    for (index, item) in ConfigItem::items().iter().enumerate() {
        let txt = (item.label)(&state);
        let rect = design::draw_text(&mut d, &txt, Align::Start, Align::Start, 20, state.ui.fg, (10, 10 + index as i32 * 20), &state.ui);
        if state.interaction.select_offset == index {
            d.draw_rectangle_lines_ex(rect, 1., state.ui.fg);
            design::draw_text(&mut d, &item.description, Align::End, Align::End, 20, state.ui.fg, (-10, -10), &state.ui);
        }
    }

    if state.interaction.select_offset + 1 < ConfigItem::items().len() && d.is_key_pressed(state.config.keybinds.down) {
        option_sfx.play();
        state.interaction.select_offset += 1;
    }

    if d.is_key_pressed(state.config.keybinds.up) {
        option_sfx.play();
        state.interaction.select_offset = state.interaction.select_offset.saturating_sub(1);
    }
    design::draw_text(&mut d, "10x - [left shift]", Align::Start, Align::End, 20, state.ui.fg, (-10, 10), &state.ui);
    design::draw_text(&mut d, "decrease - [left]", Align::Start, Align::End, 20, state.ui.fg, (-10, 30), &state.ui);
    design::draw_text(&mut d, "increase - [right]", Align::Start, Align::End, 20, state.ui.fg, (-10, 50), &state.ui);
}
