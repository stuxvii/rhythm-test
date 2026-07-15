use crate::{Screens, design, judgment::Judgment, models::*};
use raylib::prelude::*;

// ALL THE PURELY VISUAL STUFF!!
pub fn draw_ui(mut d: RaylibDrawHandle<'_>, app_state: &AppState) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(song_data) = &app_state.song_state.song_data {
        let current_visual_time = song_data.get_visual_time(app_state.song_state.song_timer, app_state.song_modifiers.sv);
        let scroll_speed = (app_state.viewport.h as f32 * app_state.config.scroll_speed) / 10.0;
        for note in song_data.notes.iter() {
            let note_visual_time = song_data.get_visual_time(note.time, app_state.song_modifiers.sv) + app_state.config.visual_offset;

            let visual_diff = note_visual_time - current_visual_time;
            let note_y = app_state.viewport.receptor_y - (visual_diff * scroll_speed) as i32;

            let note_x = *app_state.viewport.lanes.get(note.lane - 1).unwrap();
            let color = if note.state == Judgment::Miss { Color::RED } else { Color::WHITE };

            if let Some(end_time) = note.end_time {
                let end_visual_time = song_data.get_visual_time(end_time, app_state.song_modifiers.sv);
                let body_height = ((end_visual_time - note_visual_time) * scroll_speed) as i32;
                let body_y = note_y - body_height;

                d.draw_rectangle(
                    note_x.0 - app_state.ui.lane_width / 2,
                    body_y,
                    app_state.ui.lane_width,
                    body_height,
                    color,
                );
            }

            d.draw_rectangle(
                note_x.0 - app_state.ui.lane_width / 2,
                note_y - app_state.ui.note_height,
                app_state.ui.lane_width,
                app_state.ui.note_height,
                color,
            );
        }

        if let Some(last_note) = song_data.notes.last() {
            let complete_ratio = app_state.song_state.song_timer / last_note.time;
            d.draw_rectangle(
                0,
                app_state.viewport.h - app_state.ui.note_height,
                app_state.viewport.w,
                app_state.ui.note_height,
                Color::GRAY,
            );
            d.draw_rectangle(
                0,
                app_state.viewport.h - app_state.ui.note_height,
                (complete_ratio * app_state.viewport.w as f32) as i32,
                app_state.ui.note_height,
                app_state.ui.bg,
            );

            let offset = (0, 0);

            let minutes_cur_time = (app_state.song_state.song_timer as i32 / 60) % 60;
            let seconds_cur_time = app_state.song_state.song_timer as i32 % 60;
            let text_cur_time = format!("{:0>2}:{:0>2}", minutes_cur_time, seconds_cur_time);

            let minutes_rem_time = ((last_note.time - app_state.song_state.song_timer) as i32 / 60) % 60;
            let seconds_rem_time = (last_note.time - app_state.song_state.song_timer) as i32 % 60;
            let text_rem_time = format!("{:0>2}:{:0>2}", minutes_rem_time, seconds_rem_time);

            design::draw_text(
                &mut d,
                &song_data.name,
                Align::End,
                Align::Middle,
                app_state.ui.note_height,
                app_state.ui.fg,
                offset,
                &app_state.ui,
            );
            design::draw_text(
                &mut d,
                &text_rem_time,
                Align::End,
                Align::End,
                app_state.ui.note_height,
                app_state.ui.fg,
                offset,
                &app_state.ui,
            );
            design::draw_text(
                &mut d,
                &text_cur_time,
                Align::End,
                Align::Start,
                app_state.ui.note_height,
                app_state.ui.fg,
                offset,
                &app_state.ui,
            );
        }

        let precision_txt = if app_state.song_modifiers.autoplay {
            format!("AUTOPLAY")
        } else {
            format!("{:.2}%", Note::accuracy(&song_data.notes).clamp(0., 100.))
        };

        let combo_txt = format!("{}", app_state.song_state.combo);
        let judg_txt = format!("{}", Judgment::from_time(app_state.song_state.accuracy));
        if app_state.song_state.accuracy < 1. {
            let accuracy_txt = format!("{:.2}", app_state.song_state.accuracy);

            let x = design::calculate_position(&mut d, Align::Middle, Align::Middle, (0, -45));
            let opposite_color = Color::new(255 - app_state.ui.fg.r, 255 - app_state.ui.fg.g, 255 - app_state.ui.fg.b, 255);
            d.draw_poly(
                Vector2::new(x.0 as f32 + (app_state.song_state.accuracy * 100.), x.1 as f32 + 1.),
                3,
                10.,
                90.,
                opposite_color,
            );
            d.draw_poly(
                Vector2::new(x.0 as f32 + (app_state.song_state.accuracy * 100.), x.1 as f32),
                3,
                10.,
                90.,
                app_state.ui.fg,
            );

            design::draw_text(
                &mut d,
                &accuracy_txt,
                Align::Middle,
                Align::Middle,
                20,
                app_state.ui.fg,
                (0, -30),
                &app_state.ui,
            );
        }

        design::draw_text(
            &mut d,
            &precision_txt,
            Align::Start,
            Align::End,
            20,
            app_state.ui.fg,
            (-10, 10),
            &app_state.ui,
        );
        design::draw_text(
            &mut d,
            &judg_txt,
            Align::Middle,
            Align::Middle,
            30,
            app_state.ui.fg,
            (0, -15),
            &app_state.ui,
        );
        design::draw_text(
            &mut d,
            &combo_txt,
            Align::Middle,
            Align::Middle,
            20,
            app_state.ui.fg,
            (0, 20),
            &app_state.ui,
        );

        let grade_style: (String, Color) = if app_state.song_modifiers.autoplay {
            (String::from("BOT"), Color::GRAY)
        } else {
            let sh = crate::judgment::Rating::from_time(Note::accuracy(&song_data.notes).clamp(0., 100.));
            (sh.display_info().0.into(), sh.display_info().1)
        };

        design::draw_text(
            &mut d,
            &grade_style.0,
            Align::Start,
            Align::End,
            30,
            grade_style.1,
            (-10, 30),
            &app_state.ui,
        );
        let mut counts = std::collections::HashMap::new();

        for note in &song_data.notes {
            *counts.entry(note.state).or_insert(0) += 1;
        }

        let judgments = [
            Judgment::Marvelous,
            Judgment::Perfect,
            Judgment::Great,
            Judgment::Good,
            Judgment::Okay,
            Judgment::Miss,
        ];

        for (i, j) in judgments.iter().enumerate() {
            let y_off = i as i32 * 40;
            let n = counts.get(j).unwrap_or(&0);
            let text = if *n == 0 { format!("{}", j.short_form()) } else { format!("{n}") };

            let off = ((app_state.viewport.w / 2) - 40, y_off + (app_state.viewport.h / 2) - (40 * 3) + 10);
            let r = rrect(app_state.viewport.w - 80, y_off + (app_state.viewport.h / 2) - (40 * 3), 80, 40);

            d.draw_rectangle_rec(r, Color::BLACK);
            d.draw_rectangle_lines_ex(r, 1., j.color());
            design::draw_text(&mut d, &text, Align::Start, Align::Middle, 20, j.color(), off, &app_state.ui);
        }
    }
    Ok(())
}

pub fn check_inputs(d: &mut RaylibDrawHandle<'_>, app_state: &mut AppState, current_tap: &Option<Sound<'_>>) {
    let mut hitzone_color = Color::GRAY;
    let mut lane_start_pos: Vector2;
    let mut lane_end_pos: Vector2;
    if let Some(song_data) = &mut app_state.song_state.song_data {
        for (lane, (x_pos, key_code)) in app_state.viewport.lanes.iter().enumerate() {
            let acc_lane = lane + 1;
            lane_start_pos = Vector2::new(
                *x_pos as f32 - app_state.ui.lane_width as f32 / 2.,
                app_state.viewport.receptor_y as f32,
            );
            lane_end_pos = Vector2::new(
                *x_pos as f32 + app_state.ui.lane_width as f32 / 2.,
                app_state.viewport.receptor_y as f32,
            );
            if d.is_key_pressed(*key_code) {
                if let Some(accuracy) = Note::check_note_hit(
                    &mut song_data.notes,
                    acc_lane,
                    app_state.song_state.song_timer + app_state.config.input_offset,
                ) {
                    app_state.song_state.accuracy = accuracy;
                    if let Some(note) = song_data.notes.iter_mut().find(|n| {
                        n.lane == acc_lane
                            && (n.state != Judgment::None && n.state != Judgment::Miss)
                            && n.end_time.is_some()
                            && !n.is_holding
                            && (app_state.song_state.song_timer - n.time).abs() < Judgment::Good.threshold()
                    }) {
                        note.is_holding = true;
                    }

                    if Judgment::from_time(accuracy) == Judgment::Okay {
                        if app_state.song_state.combo > app_state.song_state.max_combo {
                            app_state.song_state.max_combo = app_state.song_state.combo
                        }
                        app_state.song_state.combo = 0;
                    } else if Judgment::from_time(accuracy) != Judgment::None {
                        app_state.song_state.combo += 1;
                    }
                } else {
                    app_state.song_state.accuracy = 0.;
                }

                if let Some(tap_sfx) = &current_tap {
                    tap_sfx.play();
                }
            } else if d.is_key_down(*key_code) {
                hitzone_color = Color::WHITE;
            } else {
                hitzone_color = Color::GRAY;
                for note in song_data.notes.iter_mut().filter(|n| n.is_holding && n.lane == acc_lane) {
                    let end_t = note.end_time.unwrap_or(note.time);
                    if d.is_key_up(*key_code) {
                        if app_state.song_state.song_timer < end_t - Judgment::Good.threshold() {
                            note.is_holding = false;
                            note.state = Judgment::Miss;
                            app_state.song_state.accuracy = 0.;
                            if app_state.song_state.combo > app_state.song_state.max_combo {
                                app_state.song_state.max_combo = app_state.song_state.combo
                            }
                            app_state.song_state.combo = 0;
                        } else if app_state.song_state.song_timer >= end_t {
                            note.is_holding = false;
                        }
                    }
                }
            }
            d.draw_line_ex(lane_start_pos, lane_end_pos, 10., hitzone_color);
        }
    }
}

pub fn update_music(app_state: &mut AppState, frame_time: f32, mut current_song: &Option<Music<'_>>) {
    app_state.song_state.timer += frame_time;
    if let Some(song) = &mut current_song {
        if let Some(song_data) = &app_state.song_state.song_data {
            if app_state.song_state.song_timer > 0.0 {
                if !song.is_stream_playing() {
                    song.play_stream();
                    song.seek_stream(app_state.song_state.song_timer);
                } else {
                    let last_note_time: f32 = if let Some(t) = song_data.notes.last().unwrap().end_time {
                        if t == 0. { song_data.notes.last().unwrap().time } else { t }
                    } else {
                        song_data.notes.last().unwrap().time
                    };
                    if app_state.song_state.song_timer > last_note_time {
                        app_state.current_screen = Screens::Results;
                        return;
                    } else if app_state.song_state.song_timer < song.get_time_length() {
                        song.update_stream();
                        app_state.song_state.song_timer = song.get_time_played();
                    } else {
                        app_state.song_state.song_timer += frame_time;
                    }
                }
            } else {
                app_state.song_state.song_timer = app_state.song_state.timer + (song_data.notes.get(0).unwrap().time - 3.);
            }
        }
    }
}

pub fn game_loop(
    mut d: RaylibDrawHandle<'_>,
    mut app_state: &mut AppState,
    current_tap: &Option<Sound<'_>>,
    current_song: &Option<Music<'_>>,
) -> Result<(), Box<dyn std::error::Error>> {
    // PROGRESS THE SONG AND MANAGE IT

    update_music(&mut app_state, d.get_frame_time(), current_song);
    for (x_pos, _) in app_state.viewport.lanes.clone() {
        d.draw_rectangle(
            x_pos - app_state.ui.lane_width / 2,
            0,
            app_state.ui.lane_width,
            app_state.viewport.h,
            Color::new(16, 16, 16, 255),
        );
    }

    // HERE WE DO CHECKING FOR KEY HITS AND DRAWING THE FIELD ZONE DIFFERENTLY
    check_inputs(&mut d, &mut app_state, current_tap);
    if let Some(tap_sfx) = &current_tap {
        if let Some(song_data) = &mut app_state.song_state.song_data {
            if app_state.song_modifiers.autoplay {
                for note in song_data.notes.iter_mut() {
                    if app_state.song_state.song_timer > note.time && note.state == Judgment::None {
                        note.state = Judgment::Marvelous;
                        app_state.song_state.accuracy = note.accuracy;
                        tap_sfx.play();
                        app_state.song_state.combo += 1;

                        let lane_start_pos = Vector2::new(
                            app_state.viewport.lanes[note.lane - 1].0 as f32,
                            app_state.viewport.receptor_y as f32,
                        );
                        let lane_end_pos = Vector2::new(
                            app_state.viewport.lanes[note.lane - 1].0 as f32 + app_state.ui.lane_width as f32,
                            app_state.viewport.receptor_y as f32,
                        );
                        d.draw_line_ex(lane_start_pos, lane_end_pos, 2., Color::GRAY);
                    }
                }
            } else {
                for note in song_data.notes.iter_mut() {
                    if note.is_missed(app_state.song_state.song_timer) {
                        note.state = Judgment::Miss; // since we immediately set to miss, this check wont pass the next time it's made
                        note.accuracy = 10.;
                        app_state.song_state.accuracy = note.accuracy;
                        app_state.song_state.combo = 0;
                    }
                }
            }
        }
    }

    draw_ui(d, app_state)?;
    Ok(())
}
