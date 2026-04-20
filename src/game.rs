use crate::{Screens, judgment::Judgment, models::*};
use raylib::prelude::*;

// ALL THE PURELY VISUAL STUFF!!
pub fn draw_ui(mut d: RaylibDrawHandle<'_>, app_state: &AppState) {
    if let Some(song_data) = &app_state.song_state.song_data {
        let current_visual_time = song_data.get_visual_time(app_state.song_state.song_timer);
        let scroll_speed = (app_state.viewport.h as f32 * app_state.game_config.scroll_speed) / 10.0;
        for note in app_state.song_state.notes_to_draw.iter() {
            let note_visual_time = song_data.get_visual_time(note.time) + app_state.game_config.visual_offset;

            let visual_diff = note_visual_time - current_visual_time;
            let note_y = app_state.viewport.receptor_y - (visual_diff * scroll_speed) as i32;

            let note_x = *app_state.viewport.lanes.get(note.lane - 1).unwrap();
            let color = if note.state == Judgment::Miss { Color::RED } else { Color::WHITE };

            if let Some(end_time) = note.end_time {
                let end_visual_time = song_data.get_visual_time(end_time);
                let body_height = ((end_visual_time - note_visual_time) * scroll_speed) as i32;
                let body_y = note_y - body_height;

                d.draw_rectangle(note_x.0 - app_state.ui.lane_width / 2, body_y, app_state.ui.lane_width, body_height, color);
            }

            d.draw_rectangle(
                note_x.0 - app_state.ui.lane_width / 2,
                note_y - app_state.ui.note_height,
                app_state.ui.lane_width,
                app_state.ui.note_height,
                color,
            );
        }

        if let Some(last_note) = app_state.song_state.notes_to_draw.last() {
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

            let offset = Some((0, 3));

            let minutes_cur_time = (app_state.song_state.song_timer as i32 / 60) % 60;
            let seconds_cur_time = app_state.song_state.song_timer as i32 % 60;
            let text_cur_time = format!("{:0>2}:{:0>2}", minutes_cur_time, seconds_cur_time);

            let minutes_rem_time = ((last_note.time - app_state.song_state.song_timer) as i32 / 60) % 60;
            let seconds_rem_time = (last_note.time - app_state.song_state.song_timer) as i32 % 60;
            let text_rem_time = format!("{:0>2}:{:0>2}", minutes_rem_time, seconds_rem_time);

            Align::draw_text(
                &mut d,
                &song_data.name,
                Align::End,
                Align::Middle,
                app_state.ui.note_height,
                app_state.ui.fg,
                offset,
                false,
                &app_state.ui,
            );
            Align::draw_text(
                &mut d,
                &text_rem_time,
                Align::End,
                Align::End,
                app_state.ui.note_height,
                app_state.ui.fg,
                offset,
                false,
                &app_state.ui,
            );
            Align::draw_text(
                &mut d,
                &text_cur_time,
                Align::End,
                Align::Start,
                app_state.ui.note_height,
                app_state.ui.fg,
                offset,
                false,
                &app_state.ui,
            );
        }

        let precision_txt = if app_state.game_config.autoplay {
            format!("AUTOPLAY")
        } else {
            format!("{:.2}%", Note::accuracy(&app_state.song_state.notes_to_draw).clamp(0., 100.))
        };
        let misses: Vec<&Note> = app_state.song_state.notes_to_draw.iter().filter(|n| n.state == Judgment::Miss).collect();
        let misses_txt = format!("Misses: {}", misses.len());
        let combo_txt = format!("{}", app_state.song_state.combo);
        let judg_txt = format!("{}", Judgment::from_time(app_state.song_state.accuracy));
        if app_state.song_state.accuracy < 1. {
            let accuracy_txt = format!("{:.2}", app_state.song_state.accuracy);

            let x = Align::calculate_position(&mut d, Align::Middle, Align::Middle, Some((0, -45)));
            let opposite_color = Color::new(255 - app_state.ui.fg.r, 255 - app_state.ui.fg.g, 255 - app_state.ui.fg.b, 255);
            d.draw_poly(
                Vector2::new(x.0 as f32 + (app_state.song_state.accuracy * 10.), x.1 as f32 + 1.),
                3,
                10.,
                90.,
                opposite_color,
            );
            d.draw_poly(
                Vector2::new(x.0 as f32 + (app_state.song_state.accuracy * 10.), x.1 as f32),
                3,
                10.,
                90.,
                app_state.ui.fg,
            );

            Align::draw_text(
                &mut d,
                &accuracy_txt,
                Align::Middle,
                Align::Middle,
                20,
                app_state.ui.fg,
                Some((0, -20)),
                true,
                &app_state.ui,
            );
        }
        Align::draw_text(
            &mut d,
            &precision_txt,
            Align::Start,
            Align::Middle,
            20,
            app_state.ui.fg,
            None,
            true,
            &app_state.ui,
        );
        Align::draw_text(&mut d, &misses_txt, Align::Start, Align::End, 20, app_state.ui.fg, None, false, &app_state.ui);
        Align::draw_text(&mut d, &judg_txt, Align::Middle, Align::Middle, 30, app_state.ui.fg, None, true, &app_state.ui);
        Align::draw_text(
            &mut d,
            &combo_txt,
            Align::Middle,
            Align::Middle,
            20,
            app_state.ui.fg,
            Some((0, 20)),
            true,
            &app_state.ui,
        );
    }
}

pub fn check_inputs(d: &mut RaylibDrawHandle<'_>, app_state: &mut AppState, tap_sfx: &Sound) {
    let mut hitzone_color = Color::GRAY;
    let mut lane_start_pos: Vector2;
    let mut lane_end_pos: Vector2;
    for (lane, (x_pos, key_code)) in app_state.viewport.lanes.iter().enumerate() {
        let acc_lane = lane + 1;
        lane_start_pos = Vector2::new(*x_pos as f32 - app_state.ui.lane_width as f32 / 2., app_state.viewport.receptor_y as f32);
        lane_end_pos = Vector2::new(*x_pos as f32 + app_state.ui.lane_width as f32 / 2., app_state.viewport.receptor_y as f32);
        if d.is_key_pressed(*key_code) {
            if let Some(accuracy) = Note::check_note_hit(
                &mut app_state.song_state.notes_to_draw,
                acc_lane,
                app_state.song_state.song_timer + app_state.game_config.input_offset,
            ) {
                app_state.song_state.accuracy = accuracy;
                if let Some(note) = app_state.song_state.notes_to_draw.iter_mut().find(|n| {
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

            tap_sfx.play();
        } else if d.is_key_down(*key_code) {
            hitzone_color = Color::WHITE;
        } else {
            hitzone_color = Color::GRAY;
            for note in app_state.song_state.notes_to_draw.iter_mut().filter(|n| n.is_holding && n.lane == acc_lane) {
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

pub fn update_music(app_state: &mut AppState, song: &mut Music, frame_time: f32) {
    app_state.song_state.timer += frame_time;

    if let Some(song_data) = &app_state.song_state.song_data {
        if app_state.song_state.song_timer > 0.0 {
            if !song.is_stream_playing() {
                song.play_stream();
                song.looping = false;
                song.seek_stream(app_state.song_state.song_timer);
            } else {
                let last_note_time: f32 = if let Some(t) = app_state.song_state.notes_to_draw.last().unwrap().end_time {
                    if t == 0. {
                        app_state.song_state.notes_to_draw.last().unwrap().time
                    } else {
                        t
                    }
                } else {
                    app_state.song_state.notes_to_draw.last().unwrap().time
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

pub fn game_loop(mut d: RaylibDrawHandle<'_>, mut app_state: &mut AppState, song: &mut Music<'_>, tap_sfx: &Sound<'_>) {
    // PROGRESS THE SONG AND MANAGE IT
    update_music(&mut app_state, song, d.get_frame_time());
    for (x_pos, _) in app_state.viewport.lanes.clone() {
        d.draw_rectangle(
            x_pos - app_state.ui.lane_width / 2,
            0,
            app_state.ui.lane_width,
            app_state.viewport.h,
            Color::new(16, 16, 16, 255),
        );
        d.draw_rectangle(x_pos - app_state.ui.lane_width / 2, 0, 2, app_state.viewport.h, Color::LIGHTGRAY);
    }

    // HERE WE DO CHECKING FOR KEY HITS AND DRAWING THE FIELD ZONE DIFFERENTLY
    check_inputs(&mut d, &mut app_state, &tap_sfx);

    if app_state.game_config.autoplay {
        for note in app_state.song_state.notes_to_draw.iter_mut() {
            if app_state.song_state.song_timer > note.time && note.state == Judgment::None {
                note.state = Judgment::Marvelous;
                app_state.song_state.accuracy = note.accuracy;
                tap_sfx.play();
                app_state.song_state.combo += 1;

                let lane_start_pos = Vector2::new(app_state.viewport.lanes[note.lane - 1].0 as f32, app_state.viewport.receptor_y as f32);
                let lane_end_pos = Vector2::new(
                    app_state.viewport.lanes[note.lane - 1].0 as f32 + app_state.ui.lane_width as f32,
                    app_state.viewport.receptor_y as f32,
                );
                d.draw_line_ex(lane_start_pos, lane_end_pos, 2., Color::GRAY);
            }
        }
    } else {
        for note in app_state.song_state.notes_to_draw.iter_mut() {
            if note.is_missed(app_state.song_state.song_timer) {
                note.state = Judgment::Miss; // since we immediately set to miss, this check wont pass the next time it's made
                note.accuracy = 10.;
                app_state.song_state.accuracy = note.accuracy;
                app_state.song_state.combo = 0;
            }
        }
    }

    draw_ui(d, app_state);
}
