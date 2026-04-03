use raylib::prelude::*;

use crate::{judgment::Rating, models::{Align, GameConfig, Note, ProgramState}};

pub fn draw_results(
    mut d: RaylibDrawHandle<'_>,
    in_game_state: &mut ProgramState,
    game_config: &GameConfig,
    song: &mut std::option::Option<raylib::prelude::Music<'_>>,
) {
    if let Some(song) = song {
        song.update_stream();
    }
    let total_accuracy = Note::accuracy(&in_game_state.notes_to_draw).clamp(0., 100.);

    let accuracy_txt = if game_config.autoplay {
        format!("AUTOPLAY")
    } else {
        format!("{:.2}%", total_accuracy)
    };

    let grade_style: (String, Color) = if game_config.autoplay {
        (String::from("BOT"), Color::GRAY)
    } else {
        let sh = Rating::from_time(total_accuracy);
        let color = &sh.display_info().1;
        let text = String::from(sh.display_info().0);
        (text, *color)
    };
    
    Align::draw_text(&mut d, &grade_style.0, Align::Start, Align::Start, 100, grade_style.1, Some(Vector2::new(20.,20.)), false);
    Align::draw_text(&mut d, &accuracy_txt, Align::Start, Align::Start, 40, grade_style.1, Some(Vector2::new(20.,110.)), false);
    Align::draw_text(&mut d, "Press space to go back...", Align::End, Align::Middle, 20, Color::WHITE, Some(Vector2::new(0.,-10.)), false);

    if d.is_key_pressed(KeyboardKey::KEY_SPACE) {
        *in_game_state = ProgramState::new(in_game_state.lanes, in_game_state.receptor_y);
        *song = None;
    }
}