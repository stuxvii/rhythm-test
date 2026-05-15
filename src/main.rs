use raylib::prelude::*;
mod design;
mod game;
mod judgment;
mod menu;
mod models;
mod parser;
mod results;
use crate::models::*;

pub struct ConfigItem {
    pub label: Box<dyn Fn(&AppState) -> String>,
    pub adjust: Box<dyn Fn(&mut AppState, i32)>,
}

impl ConfigItem {
    pub fn adjust(&mut self, direction: i32, state: &mut AppState) {
        if direction != 0 {
            (self.adjust)(state, direction);
        }
    }
}

fn draw_list_item(state: &AppState, d: &mut RaylibDrawHandle<'_>, visual_offset: usize, label: &str, banner: &Option<Texture2D>, is_selected: bool) -> bool {
    let mut rect = rrect(0, 0, state.viewport.w / 2, 60);
    let mut position = design::calculate_position(d, Align::Start, Align::End, (-rect.width as i32, (rect.height + 5.) as i32 * visual_offset as i32));
    rect.x = position.0 as f32;
    if is_selected {
        rect.x -= 10.;
    }
    rect.y = position.1 as f32;
    d.draw_rectangle_rec(rect, Color::DIMGRAY);
    position.1 += rect.height as i32 / 2;
    position.1 -= 5;

    design::draw_text(d, &label, Align::Start, Align::Start, 15, state.ui.fg, position, &state.ui);
    rect.width /= 2.;
    rect.x += rect.width;

    if let Some(texture) = banner {
        d.draw_texture_pro(texture, rrect(0, 0, texture.width, texture.height), rect, Vector2::new(0., 0.), 0., Color::WHITE);
    } else {
        d.draw_rectangle_gradient_ex(rect, Color::BLANK, Color::BLANK, Color::WHITE, Color::WHITE);
    }
    is_selected && d.is_key_pressed(state.config.keybinds.confirm)
}

fn main_loop() -> Result<(), Box<dyn std::error::Error>> {
    let (mut rhl, rt) = raylib::init().log_level(TraceLogLevel::LOG_NONE).resizable().height(600).width(800).msaa_4x().build();
    let mut state: AppState = AppState::init()?;

    let mut setting_items: Vec<ConfigItem> = vec![
        ConfigItem {
            label: Box::new(|s| format!("visual offset: {:.2}", s.config.visual_offset)),
            adjust: Box::new(|s, dir| s.config.visual_offset += 0.01 * dir as f32),
        },
        ConfigItem {
            label: Box::new(|s| format!("input offset: {:.2}", s.config.input_offset)),
            adjust: Box::new(|s, dir| {
                s.config.input_offset += 0.01 * dir as f32;
            }),
        },
        ConfigItem {
            label: Box::new(|s| format!("scroll speed: {:.2}", s.config.scroll_speed)),
            adjust: Box::new(|s, dir| {
                s.config.scroll_speed += 0.01 * dir as f32;
            }),
        },
        ConfigItem {
            label: Box::new(|s| format!("fps limit*: {}", s.config.max_fps)),
            adjust: Box::new(|s, dir| {
                s.config.max_fps += dir;
            }),
        },
    ];
    let audio_device: RaylibAudio = audio::RaylibAudio::init_audio_device()?;

    rhl.set_window_min_size(800, 600);
    rhl.set_target_fps(state.config.max_fps as u32);
    rhl.set_exit_key(None);

    for n in 0..5 {
        // now i know... this is ugly... but text handling is not joyous in this platform. :(
        let f = rhl.load_font_from_memory(&rt, ".ttf", include_bytes!("../lt.ttf"), 10 + (n * state.ui.text_scale), None)?;
        f.texture().set_texture_filter(&rt, TextureFilter::TEXTURE_FILTER_TRILINEAR);
        state.ui.fonts.push(f);
    }

    let mut current_song: Option<Music<'_>> = None;
    let current_tap: Option<Sound<'_>> = Some(audio_device.new_sound("./hit.wav")?);
    let mut charts: Vec<SongData> = vec![];
    let mut tried_loading_charts = false;
    let mut requested_chart_load = false;
    let mut chart_select_offset: usize = 0;
    let mut chosen_song: Option<usize> = None;
    let mut chosen_difficulty: usize = 0;
    let mut setting_element: usize = 0;

    // audio_device.set_master_volume(0.05);
    while !rhl.window_should_close() {
        let mut d = rhl.begin_drawing(&rt);
        state.viewport.h = d.get_screen_height();
        state.viewport.w = d.get_screen_width();
        d.clear_background(Color::BLANK);
        state.viewport.receptor_y = state.viewport.h - state.ui.note_height;
        if d.is_key_pressed(KeyboardKey::KEY_ESCAPE) {
            state.current_screen = Screens::StartMenu;
            state.song_state = PlayState::new();
            current_song = None;
            chosen_song = None;
            chosen_difficulty = 0;
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

                    game::game_loop(d, &mut state, &current_tap, &current_song);
                }
            }
            Screens::StartMenu => {
                let result = menu::handle_menu_screen(&mut d, &audio_device, &mut state, &mut current_song);
                if let Ok(should_break) = result
                    && should_break
                {
                    break;
                } else if let Err(err) = result {
                    return Err(err);
                }
            }
            Screens::Results => results::draw_results(d, &mut state, &current_song),
            Screens::Songs => {
                // we put this before any state assignment
                if requested_chart_load {
                    let r = parser::load_charts(&state.config.songs_path);
                    if let Ok(parsed) = r {
                        charts = parsed
                            .into_iter()
                            .map(|mut map| {
                                if let Ok(mut image) = Image::load_image(&map.banner) {
                                    image.resize(state.viewport.w / 2, state.ui.song_item_height);
                                    if let Ok(txt) = d.load_texture_from_image(&rt, &image) {
                                        map.banner_texture = Some(txt);
                                    }
                                    std::mem::drop(image);
                                }
                                map
                            })
                            .collect();
                        charts.sort_by(|a, b| a.name.to_lowercase().cmp(&b.name.to_lowercase()));
                    } else if let Err(err) = r {
                        return Err(format!("Couldn't load charts, check the folder path in config.json, error is: {err}").into());
                    }
                    tried_loading_charts = true;
                }

                // but then change it for the next frame
                requested_chart_load = d.is_key_pressed(KeyboardKey::KEY_F5) || !tried_loading_charts;

                // to let the following message to draw
                if requested_chart_load {
                    design::draw_text(&mut d, "Please wait...", Align::Middle, Align::Middle, 20, state.ui.fg, (0, 0), &state.ui);
                }

                if charts.is_empty() && tried_loading_charts && !requested_chart_load {
                    design::draw_text(&mut d, "No charts!", Align::Middle, Align::Middle, 20, state.ui.fg, (0, 0), &state.ui);
                    design::draw_text(&mut d, "F5 to refresh", Align::Middle, Align::Middle, 15, state.ui.fg, (0, 20), &state.ui);
                    design::draw_text(&mut d, "ESC to go back", Align::Middle, Align::Middle, 15, state.ui.fg, (0, -20), &state.ui);
                } else {
                    if d.is_key_pressed(state.config.keybinds.back) {
                        chosen_song = None;
                        chosen_difficulty = 0;
                    }
                    if d.is_key_pressed_repeat(state.config.keybinds.left) || d.is_key_pressed(state.config.keybinds.left) {
                        if state.song_modifiers.speed >= 0.05 {
                            state.song_modifiers.speed -= 0.05;
                        }
                    }
                    if d.is_key_pressed_repeat(state.config.keybinds.right) || d.is_key_pressed(state.config.keybinds.right) {
                        state.song_modifiers.speed += 0.05;
                    }
                    if let Some(data) = charts.get(chart_select_offset) {
                        design::draw_text(&mut d, &data.name, Align::Start, Align::Start, 15, state.ui.fg, (10, 10), &state.ui);
                        if let Some(data) = data.difficulties.get(chosen_difficulty) {
                            for (idx, text) in data.hints.iter().enumerate() {
                                design::draw_text(&mut d, &text, Align::Start, Align::Start, 15, state.ui.fg, (10, 25 + (idx as i32 * 15)), &state.ui);
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

                    if let Some(data) = chosen_song.and_then(|idx| charts.get(idx)) {
                        let hints = [("[confirm] - go.", -70), ("hold [ctrl] - no sv", -50), ("hold [shift] - autoplay", -30)];
                        for (text, y_off) in hints {
                            design::draw_text(&mut d, text, Align::End, Align::Start, 20, state.ui.fg, (10, y_off), &state.ui);
                        }

                        for (v_off, (idx, song_data)) in data.difficulties.iter().enumerate().skip(chosen_difficulty).take(20).enumerate() {
                            if draw_list_item(&state, &mut d, v_off, &song_data.metadata.name, &data.banner_texture, idx == chosen_difficulty) {
                                let mut music = audio_device.new_music(&song_data.metadata.song)?;
                                music.looping = false;
                                music.set_pitch(state.song_modifiers.speed);
                                current_song = Some(music);
                                state.song_modifiers.sv = !d.is_key_down(KeyboardKey::KEY_LEFT_CONTROL);
                                state.song_modifiers.autoplay = d.is_key_down(KeyboardKey::KEY_LEFT_SHIFT);
                                state.song_state.song_data = Some(song_data.clone());
                                state.current_screen = Screens::Game;
                            }
                        }

                        if nav_down && chosen_difficulty < data.difficulties.len() - 1 {
                            chosen_difficulty += 1;
                        }
                        if nav_up {
                            chosen_difficulty = chosen_difficulty.saturating_sub(1);
                        }
                    } else {
                        for (v_off, (idx, chart)) in charts.iter().enumerate().skip(chart_select_offset).take(20).enumerate() {
                            if draw_list_item(&state, &mut d, v_off, &chart.name, &chart.banner_texture, idx == chart_select_offset) {
                                chosen_song = Some(idx);
                            }
                        }

                        if nav_down && chart_select_offset < charts.len() - 1 {
                            chart_select_offset += 1;
                        }
                        if nav_up {
                            chart_select_offset = chart_select_offset.saturating_sub(1);
                        }
                    }
                }
            }
            Screens::Settings => {
                if let Some(item) = setting_items.get_mut(setting_element) {
                    let nav_down = d.is_key_pressed_repeat(state.config.keybinds.left) || d.is_key_pressed(state.config.keybinds.left);
                    let nav_up = d.is_key_pressed_repeat(state.config.keybinds.right) || d.is_key_pressed(state.config.keybinds.right);
                    let mut delta = if nav_down {-1} else if nav_up {1} else {0};
                    if d.is_key_down(KeyboardKey::KEY_LEFT_SHIFT) {
                        delta *= 10;
                    }
                    item.adjust(delta, &mut state);
                }

                for (index, item) in setting_items.iter_mut().enumerate() {
                    let txt = (item.label)(&state);
                    let rect = design::draw_text(&mut d, &txt, Align::Start, Align::Start, 20, state.ui.fg, (10, 10 + index as i32 * 20), &state.ui);
                    if setting_element == index {
                        d.draw_rectangle_lines_ex(rect, 1., state.ui.fg);
                    }
                }

                if setting_element + 1 < setting_items.len() && d.is_key_pressed(state.config.keybinds.down) {
                    setting_element += 1;
                }

                if d.is_key_pressed(state.config.keybinds.up) {
                    setting_element = setting_element.saturating_sub(1);
                }

                design::draw_text(
                    &mut d,
                    "Navigation keybinds\n[left]/[down]/[up]/[right] - D/F/J/K\n[confirm] - Z\n[back] - X\n\nGlobal keybinds\n[esc] - main menu",
                    Align::End,
                    Align::Start,
                    20,
                    state.ui.fg,
                    (10, -10),
                    &state.ui,
                );
                design::draw_text(&mut d, "* - restart to apply", Align::End, Align::End, 20, state.ui.fg, (-10, -10), &state.ui);
                design::draw_text(&mut d, "if the FPS setting is 0 or less", Align::End, Align::End, 20, state.ui.fg, (-10, -30), &state.ui);
                design::draw_text(&mut d, "the framerate will be unlocked", Align::End, Align::End, 20, state.ui.fg, (-10, -50), &state.ui);
                design::draw_text(&mut d, "10x - [left shift]", Align::Start, Align::End, 20, state.ui.fg, (-10, 10), &state.ui);
                design::draw_text(&mut d, "decrease - [left]", Align::Start, Align::End, 20, state.ui.fg, (-10, 30), &state.ui);
                design::draw_text(&mut d, "increase - [right]", Align::Start, Align::End, 20, state.ui.fg, (-10, 50), &state.ui);
            }
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
