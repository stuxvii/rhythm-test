use std::error::Error;

use crate::{
    design::{self, *},
    models::*,
    parser,
};
use raylib::prelude::*;
pub fn draw_songs<'a>(
    mut d: RaylibDrawHandle<'_>,
    state: &mut AppState,
    soundscape: &mut Soundscape<'a>,
    audio_device: &'a RaylibAudio,
    rt: &RaylibThread,
) -> Result<(), Box<dyn Error>> {
    // we put this before any state assignment
    if state.interaction.requested_chart_load {
        let r = parser::load_charts(&state.config.songs_path);
        if let Ok(parsed) = r {
            if state.config.load_images {
                state.charts = parsed
                    .into_iter()
                    .map(|mut map| {
                        if let Ok(mut image) = Image::load_image(&map.banner) {
                            image.resize(state.viewport.w / 2, state.ui.song_item_height);
                            if let Ok(txt) = d.load_texture_from_image(&rt, &image) {
                                map.banner_texture = Some(txt);
                            }
                        }
                        if let Ok(image) = Image::load_image(&map.background) {
                            if let Ok(txt) = d.load_texture_from_image(&rt, &image) {
                                map.background_texture = Some(txt);
                            }
                        }
                        map
                    })
                    .collect();
            } else {
                state.charts = parsed;
            }
            state.charts.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
        } else if let Err(err) = r {
            return Err(format!("Couldn't load charts, check the folder path in config.json, error is: {err}").into());
        }
        state.interaction.tried_loading_charts = true;
    }

    if let Some(data) = state.charts.get(state.interaction.select_offset) {
        if let Some(txt) = &data.background_texture {
            let rect = rrect(0, 0, txt.width, txt.height);
            let sc_size = Vector2::new(state.viewport.w as f32, state.viewport.h as f32);
            let dest_rec = center_crop_fit(Vector2::new(txt.width as f32, txt.height as f32), sc_size);
            d.draw_texture_pro(txt, rect, dest_rec, Vector2::new(0., 0.), 0., Color::GRAY);
        }
    }
    // but then change it for the next frame
    state.interaction.requested_chart_load = d.is_key_pressed(KeyboardKey::KEY_F5) || !state.interaction.tried_loading_charts;

    if state.charts.is_empty() && state.interaction.tried_loading_charts && !state.interaction.requested_chart_load {
        design::draw_text(
            &mut d,
            "No charts loaded!",
            Align::Middle,
            Align::Middle,
            30,
            state.ui.fg,
            (0, -5),
            &state.ui,
        );
        design::draw_text(
            &mut d,
            "F5 to refresh",
            Align::Middle,
            Align::Middle,
            20,
            state.ui.fg,
            (0, 30),
            &state.ui,
        );
        design::draw_text(
            &mut d,
            "ESC to go back",
            Align::Middle,
            Align::Middle,
            20,
            state.ui.fg,
            (0, -20),
            &state.ui,
        );
    } else {
        if d.is_key_pressed(state.config.keybinds.back) {
            state.interaction.chosen_song = None;
            state.interaction.chosen_difficulty = 0;
            soundscape.cancel_sfx.play();
        }
        if d.is_key_pressed_repeat(state.config.keybinds.left) || d.is_key_pressed(state.config.keybinds.left) {
            if state.song_modifiers.speed >= 0.05 {
                state.song_modifiers.speed -= 0.05;
            }
        }
        if d.is_key_pressed_repeat(state.config.keybinds.right) || d.is_key_pressed(state.config.keybinds.right) {
            state.song_modifiers.speed += 0.05;
        }
        if let Some(data) = state.charts.get(state.interaction.select_offset) {
            design::draw_text(&mut d, &data.name, Align::Start, Align::Start, 15, state.ui.fg, (10, 10), &state.ui);
            if let Some(data) = data.difficulties.get(state.interaction.chosen_difficulty) {
                for (idx, text) in data.hints.iter().enumerate() {
                    design::draw_text(
                        &mut d,
                        &text,
                        Align::Start,
                        Align::Start,
                        15,
                        state.ui.fg,
                        (10, 25 + (idx as i32 * 15)),
                        &state.ui,
                    );
                }
                if let Some(first_note) = data.notes.first() {
                    if let Some(last_note) = data.notes.last() {
                        let txt = format!("length: {}s", last_note.time - first_note.time);
                        design::draw_text(&mut d, &txt, Align::Start, Align::Start, 15, state.ui.fg, (10, 70), &state.ui);
                    }
                }
            }

            let txt = format!("[left] speed: {:.2}x [right]", state.song_modifiers.speed);
            design::draw_text(&mut d, &txt, Align::End, Align::Start, 20, state.ui.fg, (10, -10), &state.ui);
        }
        let nav_down = d.is_key_pressed_repeat(state.config.keybinds.down) || d.is_key_pressed(state.config.keybinds.down);
        let nav_up = d.is_key_pressed_repeat(state.config.keybinds.up) || d.is_key_pressed(state.config.keybinds.up);

        if let Some(data) = state.interaction.chosen_song.and_then(|idx| state.charts.get(idx)) {
            let hints = [
                ("[confirm] - go.", -70),
                ("hold [ctrl] - no sv", -50),
                ("hold [shift] - autoplay", -30),
            ];
            for (text, y_off) in hints {
                design::draw_text(&mut d, text, Align::End, Align::Start, 20, state.ui.fg, (10, y_off), &state.ui);
            }

            for (v_off, (idx, song_data)) in data
                .difficulties
                .iter()
                .enumerate()
                .skip(state.interaction.chosen_difficulty)
                .take(20)
                .enumerate()
            {
                if draw_list_item(
                    &state,
                    &mut d,
                    v_off,
                    &song_data.metadata.name,
                    &data.banner_texture,
                    idx == state.interaction.chosen_difficulty,
                ) {
                    let mut music = audio_device.new_music(&song_data.metadata.song)?;
                    music.set_looping(false);
                    music.set_pitch(state.song_modifiers.speed);
                    soundscape.current_song = Some(music);
                    state.song_modifiers.sv = !d.is_key_down(KeyboardKey::KEY_LEFT_CONTROL);
                    state.song_modifiers.autoplay = d.is_key_down(KeyboardKey::KEY_LEFT_SHIFT);
                    state.song_state.song_data = Some(song_data.clone());
                    state.current_screen = Screens::Game;
                }
            }

            if nav_down && state.interaction.chosen_difficulty < data.difficulties.len() - 1 {
                state.interaction.chosen_difficulty += 1;
                soundscape.option_sfx.play();
            }
            if nav_up {
                let s = state.interaction.chosen_difficulty.saturating_sub(1);
                if s != state.interaction.chosen_difficulty {
                    soundscape.option_sfx.play();
                }
                state.interaction.chosen_difficulty = state.interaction.chosen_difficulty.saturating_sub(1);
            }
        } else {
            for (v_off, (idx, chart)) in state
                .charts
                .iter()
                .enumerate()
                .skip(state.interaction.select_offset)
                .take(20)
                .enumerate()
            {
                if draw_list_item(
                    &state,
                    &mut d,
                    v_off,
                    &chart.name,
                    &chart.banner_texture,
                    idx == state.interaction.select_offset,
                ) {
                    soundscape.option_sfx.play();
                    state.interaction.chosen_song = Some(idx);
                }
            }

            if nav_down && state.interaction.select_offset < state.charts.len() - 1 {
                state.interaction.select_offset += 1;
                soundscape.option_sfx.play();
            }
            if nav_up {
                let s = state.interaction.select_offset.saturating_sub(1);
                if s != state.interaction.select_offset {
                    soundscape.option_sfx.play();
                }
                state.interaction.select_offset = s;
            }
        }
    }

    // to let the following message to draw
    if state.interaction.requested_chart_load {
        design::draw_text(
            &mut d,
            "Please wait...",
            Align::Middle,
            Align::Middle,
            20,
            state.ui.fg,
            (0, 0),
            &state.ui,
        );
    }

    Ok(())
}
