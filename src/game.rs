use crate::{BG_COLOR, FG_COLOR, LANE_WIDTH, NOTE_HEIGHT, ProgramState, ScreenDimension, Screens, judgment::Judgment, models::*};
use raylib::prelude::*;

pub fn game_loop(
    mut d: RaylibDrawHandle<'_>,
    screen_dimensions: ScreenDimension,
    in_game_state: &mut ProgramState,
    song: &mut Music,
    tap_sfx: &Sound,
    game_config: &GameConfig,
) {
    let song_data = in_game_state.song_data.clone().unwrap();
    // PROGRESS THE SONG AND MANAGE IT
    if in_game_state.current_song_timer > 0.0 {
        
        if !song.is_stream_playing() {
            song.play_stream();
            song.looping = false;
            song.seek_stream(song_data.offset);
        } else {
            if in_game_state.current_song_timer > song.get_time_length() {
                in_game_state.current_screen = Screens::Results;
                return;
            } else if in_game_state.current_song_timer < song.get_time_length() {
                song.update_stream();
                in_game_state.current_song_timer = song.get_time_played();
            } else {
                in_game_state.current_song_timer += d.get_frame_time();
            }
        }
    } else {
        in_game_state.current_song_timer = in_game_state.current_timer + (song_data.offset);
    }

    in_game_state.current_timer += d.get_frame_time();

    for (x_pos, _) in in_game_state.lanes {
        d.draw_rectangle(x_pos, 0, LANE_WIDTH, screen_dimensions.h, Color::new(50, 25, 50, 255));
        d.draw_rectangle(x_pos, 0, 2, screen_dimensions.h, Color::new(255, 255, 255, 64));
    }

    // HERE WE DO CHECKING FOR KEY HITS AND DRAWING THE FIELD ZONE DIFFERENTLY
    for (lane, (x_pos, key_code)) in in_game_state.lanes.iter().enumerate() {
        let acc_lane = lane + 1;
        let lane_start_pos = Vector2::new(*x_pos as f32, in_game_state.receptor_y as f32);
        let lane_end_pos = Vector2::new(*x_pos as f32 + LANE_WIDTH as f32, in_game_state.receptor_y as f32);
        if d.is_key_pressed(*key_code) {
            in_game_state.cur_judge = Note::check_note_hit(&mut in_game_state.notes_to_draw, acc_lane, in_game_state.current_song_timer);

            if let Some(note) = in_game_state.notes_to_draw.iter_mut().find(|n| {
                n.lane == acc_lane
                    && (n.state != Judgment::None && n.state != Judgment::Miss)
                    && n.end_time.is_some()
                    && !n.is_holding
                    && (in_game_state.current_song_timer - n.time).abs() < Judgment::Okay.threshold()
            }) {
                note.is_holding = true;
            }

            if in_game_state.cur_judge == Judgment::Ehhh {
                in_game_state.combo = 0;
            } else if in_game_state.cur_judge != Judgment::None {
                in_game_state.combo += 1;
            }

            d.draw_line_ex(lane_start_pos, lane_end_pos, 10., Color::WHITE);
            tap_sfx.play();
        } else if d.is_key_down(*key_code) {
            d.draw_line_ex(lane_start_pos, lane_end_pos, 10., Color::LIGHTGRAY);
        } else {
            d.draw_line_ex(lane_start_pos, lane_end_pos, 10., Color::GRAY);
            for note in in_game_state.notes_to_draw.iter_mut().filter(|n| n.is_holding && n.lane == acc_lane) {
                let end_t = note.end_time.unwrap_or(note.time);
                if d.is_key_up(*key_code) {
                    if in_game_state.current_song_timer < end_t - Judgment::Okay.threshold() {
                        note.is_holding = false;
                        note.state = Judgment::Miss;
                        in_game_state.cur_judge = Judgment::Miss;
                        in_game_state.combo = 0;
                    } else if in_game_state.current_song_timer >= end_t {
                        note.is_holding = false;
                    }
                }
            }
        }
    }

    if game_config.autoplay {
        for note in in_game_state.notes_to_draw.iter_mut() {
            if in_game_state.current_song_timer > note.time && note.state == Judgment::None {
                note.state = Judgment::Perfect;
                in_game_state.cur_judge = note.state;
                tap_sfx.play();
                in_game_state.combo += 1;

                let lane_start_pos = Vector2::new(in_game_state.lanes[note.lane - 1].0 as f32, in_game_state.receptor_y as f32);
                let lane_end_pos = Vector2::new(in_game_state.lanes[note.lane - 1].0 as f32 + LANE_WIDTH as f32, in_game_state.receptor_y as f32);
                d.draw_line_ex(lane_start_pos, lane_end_pos, 2., Color::WHITE);
            }
        }
    } else {
        for note in in_game_state.notes_to_draw.iter_mut() {
            if note.is_missed(in_game_state.current_song_timer) {
                note.state = Judgment::Miss; // since we immediately set to miss, this check wont pass the next time it's made
                note.accuracy = 10.;
                in_game_state.cur_judge = note.state;
                in_game_state.combo = 0;
            }
        }
    }

    // ALL THE PURELY VISUAL STUFF!!
    for note in in_game_state.notes_to_draw.iter() {
        let time_diff = note.time - in_game_state.current_song_timer;
        let scroll_factor = (screen_dimensions.h as f32 * game_config.scroll_speed) / 50.;

        let note_y = in_game_state.receptor_y - NOTE_HEIGHT - (time_diff * (song_data.bpm / 60.0) * scroll_factor) as i32;

        let note_x = in_game_state.lanes[note.lane - 1].0;
        let color = if note.state == Judgment::Miss { Color::RED } else { Color::LIGHTBLUE };

        if let Some(end_time) = note.end_time {
            let duration_beats = (end_time - note.time) * (song_data.bpm / 60.0);
            let body_height = (duration_beats * scroll_factor) as i32;
            let body_y = note_y - body_height;

            d.draw_rectangle(note_x, body_y, LANE_WIDTH, body_height, color);
        }

        d.draw_rectangle(note_x, note_y, LANE_WIDTH, NOTE_HEIGHT, color);
    }

    if let Some(last_note) = in_game_state.notes_to_draw.last() {
        let complete_ratio = in_game_state.current_song_timer / last_note.time;
        d.draw_rectangle(0, screen_dimensions.h - NOTE_HEIGHT, screen_dimensions.w, NOTE_HEIGHT, Color::GRAY);
        d.draw_rectangle(
            0,
            screen_dimensions.h - NOTE_HEIGHT,
            (complete_ratio * screen_dimensions.w as f32) as i32,
            NOTE_HEIGHT,
            BG_COLOR,
        );

        let offset = Some(Vector2::new(0., 3.));

        let minutes_cur_time = (in_game_state.current_song_timer as i32 / 60) % 60;
        let seconds_cur_time = in_game_state.current_song_timer as i32 % 60;
        let text_cur_time = format!("{:0>2}:{:0>2}", minutes_cur_time, seconds_cur_time);

        let minutes_rem_time = ((last_note.time - in_game_state.current_song_timer) as i32 / 60) % 60;
        let seconds_rem_time = (last_note.time - in_game_state.current_song_timer) as i32 % 60;
        let text_rem_time = format!("{:0>2}:{:0>2}", minutes_rem_time, seconds_rem_time);

        Align::draw_text(&mut d, &song_data.name, Align::End, Align::Middle, NOTE_HEIGHT, FG_COLOR, offset);
        Align::draw_text(&mut d, &text_rem_time, Align::End, Align::End, NOTE_HEIGHT, FG_COLOR, offset);
        Align::draw_text(&mut d, &text_cur_time, Align::End, Align::Start, NOTE_HEIGHT, FG_COLOR, offset);
    }

    let accuracy_txt = if game_config.autoplay {
        format!("AUTOPLAY")
    } else {
        format!("{:.2}%", Note::accuracy(&in_game_state.notes_to_draw).clamp(0., 100.))
    };
    let misses: Vec<&Note> = in_game_state.notes_to_draw.iter().filter(|n| n.state == Judgment::Miss).collect();
    let misses_txt = format!("Misses: {}", misses.len());
    let combo_txt = format!("{}", in_game_state.combo);
    let judg_txt = format!("{}", in_game_state.cur_judge);

    Align::draw_text(&mut d, &accuracy_txt, Align::Start, Align::Middle, 20, BG_COLOR, None);
    Align::draw_text(&mut d, &misses_txt, Align::Start, Align::End, 20, BG_COLOR, None);
    Align::draw_text(&mut d, &combo_txt, Align::Middle, Align::Middle, 20, BG_COLOR, Some(Vector2::new(0., 20.)));
    Align::draw_text(&mut d, &judg_txt, Align::Middle, Align::Middle, 30, BG_COLOR, None);
}
