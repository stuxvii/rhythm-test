use crate::{ProgramState, ScreenDimension, Screens, UIElements, judgment::Judgment, models::*};
use raylib::prelude::*;

// ALL THE PURELY VISUAL STUFF!!
pub fn draw_ui(
    mut d: RaylibDrawHandle<'_>,
    screen_dimensions: ScreenDimension,
    in_game_state: &mut ProgramState,
    game_config: &GameConfig,
    song_data: &SongData,
    ui_state: &UIElements,
) {
    let current_visual_time = song_data.get_visual_time(in_game_state.current_song_timer);
    let scroll_speed = (screen_dimensions.h as f32 * game_config.scroll_speed) / 10.0;
    for note in in_game_state.notes_to_draw.iter() {
        let note_visual_time = song_data.get_visual_time(note.time) + game_config.visual_offset;

        let visual_diff = note_visual_time - current_visual_time;
        let note_y = in_game_state.receptor_y - (visual_diff * scroll_speed) as i32;

        let note_x = *in_game_state.lanes.get(note.lane - 1).unwrap();
        let color = if note.state == Judgment::Miss { Color::RED } else { Color::WHITE };

        if let Some(end_time) = note.end_time {
            let end_visual_time = song_data.get_visual_time(end_time);
            let body_height = ((end_visual_time - note_visual_time) * scroll_speed) as i32;
            let body_y = note_y - body_height;

            d.draw_rectangle(note_x.0 - ui_state.lane_width / 2, body_y, ui_state.lane_width, body_height, color);
        }

        d.draw_rectangle(
            note_x.0 - ui_state.lane_width / 2,
            note_y - ui_state.note_height,
            ui_state.lane_width,
            ui_state.note_height,
            color,
        );
    }

    if let Some(last_note) = in_game_state.notes_to_draw.last() {
        let complete_ratio = in_game_state.current_song_timer / last_note.time;
        d.draw_rectangle(
            0,
            screen_dimensions.h - ui_state.note_height,
            screen_dimensions.w,
            ui_state.note_height,
            Color::GRAY,
        );
        d.draw_rectangle(
            0,
            screen_dimensions.h - ui_state.note_height,
            (complete_ratio * screen_dimensions.w as f32) as i32,
            ui_state.note_height,
            ui_state.bg_color,
        );

        let offset = Some((0, 3));

        let minutes_cur_time = (in_game_state.current_song_timer as i32 / 60) % 60;
        let seconds_cur_time = in_game_state.current_song_timer as i32 % 60;
        let text_cur_time = format!("{:0>2}:{:0>2}", minutes_cur_time, seconds_cur_time);

        let minutes_rem_time = ((last_note.time - in_game_state.current_song_timer) as i32 / 60) % 60;
        let seconds_rem_time = (last_note.time - in_game_state.current_song_timer) as i32 % 60;
        let text_rem_time = format!("{:0>2}:{:0>2}", minutes_rem_time, seconds_rem_time);

        Align::draw_text(
            &mut d,
            &song_data.name,
            Align::End,
            Align::Middle,
            ui_state.note_height,
            ui_state.fg_color,
            offset,
            false,
            &ui_state,
        );
        Align::draw_text(
            &mut d,
            &text_rem_time,
            Align::End,
            Align::End,
            ui_state.note_height,
            ui_state.fg_color,
            offset,
            false,
            &ui_state,
        );
        Align::draw_text(
            &mut d,
            &text_cur_time,
            Align::End,
            Align::Start,
            ui_state.note_height,
            ui_state.fg_color,
            offset,
            false,
            &ui_state,
        );
    }

    let precision_txt = if game_config.autoplay {
        format!("AUTOPLAY")
    } else {
        format!("{:.2}%", Note::accuracy(&in_game_state.notes_to_draw).clamp(0., 100.))
    };
    let misses: Vec<&Note> = in_game_state.notes_to_draw.iter().filter(|n| n.state == Judgment::Miss).collect();
    let misses_txt = format!("Misses: {}", misses.len());
    let combo_txt = format!("{}", in_game_state.combo);
    let judg_txt = format!("{}", Judgment::from_time(in_game_state.current_accuracy));
    if in_game_state.current_accuracy < 1. {
        let accuracy_txt = format!("{:.2}", in_game_state.current_accuracy);

        let x = Align::calculate_position(&mut d, Align::Middle, Align::Middle, Some((0, -45)));
        let opposite_color = Color::new(255 - ui_state.fg_color.r, 255 - ui_state.fg_color.g, 255 - ui_state.fg_color.b, 255);
        d.draw_poly(
            Vector2::new(x.0 as f32 + (in_game_state.current_accuracy * 10.), x.1 as f32 + 1.),
            3,
            10.,
            90.,
            opposite_color,
        );
        d.draw_poly(
            Vector2::new(x.0 as f32 + (in_game_state.current_accuracy * 10.), x.1 as f32),
            3,
            10.,
            90.,
            ui_state.fg_color,
        );

        Align::draw_text(
            &mut d,
            &accuracy_txt,
            Align::Middle,
            Align::Middle,
            20,
            ui_state.fg_color,
            Some((0, -20)),
            true,
            &ui_state,
        );
    }
    Align::draw_text(
        &mut d,
        &precision_txt,
        Align::Start,
        Align::Middle,
        20,
        ui_state.fg_color,
        None,
        true,
        &ui_state,
    );
    Align::draw_text(&mut d, &misses_txt, Align::Start, Align::End, 20, ui_state.fg_color, None, false, &ui_state);
    Align::draw_text(&mut d, &judg_txt, Align::Middle, Align::Middle, 30, ui_state.fg_color, None, true, &ui_state);
    Align::draw_text(
        &mut d,
        &combo_txt,
        Align::Middle,
        Align::Middle,
        20,
        ui_state.fg_color,
        Some((0, 20)),
        true,
        &ui_state,
    );
}

pub fn check_inputs(d: &mut RaylibDrawHandle<'_>, in_game_state: &mut ProgramState, tap_sfx: &Sound, game_config: &GameConfig, ui_state: &UIElements) {
    let mut hitzone_color = Color::GRAY;
    let mut lane_start_pos: Vector2;
    let mut lane_end_pos: Vector2;
    for (lane, (x_pos, key_code)) in in_game_state.lanes.iter().enumerate() {
        let acc_lane = lane + 1;
        lane_start_pos = Vector2::new(*x_pos as f32 - ui_state.lane_width as f32 / 2., in_game_state.receptor_y as f32);
        lane_end_pos = Vector2::new(*x_pos as f32 + ui_state.lane_width as f32 / 2., in_game_state.receptor_y as f32);
        if d.is_key_pressed(*key_code) {
            if let Some(accuracy) = Note::check_note_hit(
                &mut in_game_state.notes_to_draw,
                acc_lane,
                in_game_state.current_song_timer + game_config.input_offset,
            ) {
                in_game_state.current_accuracy = accuracy;
                if let Some(note) = in_game_state.notes_to_draw.iter_mut().find(|n| {
                    n.lane == acc_lane
                        && (n.state != Judgment::None && n.state != Judgment::Miss)
                        && n.end_time.is_some()
                        && !n.is_holding
                        && (in_game_state.current_song_timer - n.time).abs() < Judgment::Good.threshold()
                }) {
                    note.is_holding = true;
                }

                if Judgment::from_time(accuracy) == Judgment::Okay {
                    in_game_state.combo = 0;
                } else if Judgment::from_time(accuracy) != Judgment::None {
                    in_game_state.combo += 1;
                }
            } else {
                in_game_state.current_accuracy = 0.;
            }

            tap_sfx.play();
        } else if d.is_key_down(*key_code) {
            hitzone_color = Color::WHITE;
        } else {
            hitzone_color = Color::GRAY;
            for note in in_game_state.notes_to_draw.iter_mut().filter(|n| n.is_holding && n.lane == acc_lane) {
                let end_t = note.end_time.unwrap_or(note.time);
                if d.is_key_up(*key_code) {
                    if in_game_state.current_song_timer < end_t - Judgment::Good.threshold() {
                        note.is_holding = false;
                        note.state = Judgment::Miss;
                        in_game_state.current_accuracy = 0.;
                        in_game_state.combo = 0;
                    } else if in_game_state.current_song_timer >= end_t {
                        note.is_holding = false;
                    }
                }
            }
        }
        d.draw_line_ex(lane_start_pos, lane_end_pos, 10., hitzone_color);
    }
}

pub fn update_music(in_game_state: &mut ProgramState, song: &mut Music, frame_time: f32) {
    in_game_state.current_timer += frame_time;

    if let Some(song_data) = &in_game_state.song_data {
        if in_game_state.current_song_timer > 0.0 {
            if !song.is_stream_playing() {
                song.play_stream();
                song.looping = false;
                song.seek_stream(in_game_state.current_song_timer);
            } else {
                let last_note_time: f32 = if let Some(t) = in_game_state.notes_to_draw.last().unwrap().end_time {
                    if t == 0. { in_game_state.notes_to_draw.last().unwrap().time } else { t }
                } else {
                    in_game_state.notes_to_draw.last().unwrap().time
                };
                if in_game_state.current_song_timer > last_note_time {
                    in_game_state.current_screen = Screens::Results;
                    return;
                } else if in_game_state.current_song_timer < song.get_time_length() {
                    song.update_stream();
                    in_game_state.current_song_timer = song.get_time_played();
                } else {
                    in_game_state.current_song_timer += frame_time;
                }
            }
        } else {
            in_game_state.current_song_timer = in_game_state.current_timer + (song_data.notes.get(0).unwrap().time - 3.);
        }
    }
}

pub fn game_loop(
    mut d: RaylibDrawHandle<'_>,
    screen_dimensions: ScreenDimension,
    in_game_state: &mut ProgramState,
    song: &mut Music,
    tap_sfx: &Sound,
    game_config: &GameConfig,
    ui_state: &UIElements,
) {
    // PROGRESS THE SONG AND MANAGE IT
    update_music(in_game_state, song, d.get_frame_time());
    for (x_pos, _) in in_game_state.lanes.clone() {
        d.draw_rectangle(
            x_pos - ui_state.lane_width / 2,
            0,
            ui_state.lane_width,
            screen_dimensions.h,
            Color::new(16, 16, 16, 255),
        );
        d.draw_rectangle(x_pos - ui_state.lane_width / 2, 0, 2, screen_dimensions.h, Color::LIGHTGRAY);
    }

    // HERE WE DO CHECKING FOR KEY HITS AND DRAWING THE FIELD ZONE DIFFERENTLY
    check_inputs(&mut d, in_game_state, tap_sfx, game_config, ui_state);

    if game_config.autoplay {
        for note in in_game_state.notes_to_draw.iter_mut() {
            if in_game_state.current_song_timer > note.time && note.state == Judgment::None {
                note.state = Judgment::Marvelous;
                in_game_state.current_accuracy = note.accuracy;
                tap_sfx.play();
                in_game_state.combo += 1;

                let lane_start_pos = Vector2::new(in_game_state.lanes[note.lane - 1].0 as f32, in_game_state.receptor_y as f32);
                let lane_end_pos = Vector2::new(
                    in_game_state.lanes[note.lane - 1].0 as f32 + ui_state.lane_width as f32,
                    in_game_state.receptor_y as f32,
                );
                d.draw_line_ex(lane_start_pos, lane_end_pos, 2., Color::GRAY);
            }
        }
    } else {
        for note in in_game_state.notes_to_draw.iter_mut() {
            if note.is_missed(in_game_state.current_song_timer) {
                note.state = Judgment::Miss; // since we immediately set to miss, this check wont pass the next time it's made
                note.accuracy = 10.;
                in_game_state.current_accuracy = note.accuracy;
                in_game_state.combo = 0;
            }
        }
    }

    draw_ui(
        d,
        screen_dimensions,
        in_game_state,
        game_config,
        &in_game_state.song_data.clone().unwrap(),
        &ui_state,
    );
}
