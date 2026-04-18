use raylib::prelude::*;

use crate::{
    UIElements, judgment::{Judgment, Rating}, models::{Align, GameConfig, Note, ProgramState}
};

pub fn draw_results(
    mut d: RaylibDrawHandle<'_>,
    in_game_state: &mut ProgramState,
    game_config: &GameConfig,
    song: &mut std::option::Option<raylib::prelude::Music<'_>>,
    ui_state: &UIElements
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
        let color = sh.display_info().1;
        let text = String::from(sh.display_info().0);
        (text, color)
    };

    use std::collections::HashMap;

    let mut counts = HashMap::new();
    for note in &in_game_state.notes_to_draw {
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

    Align::draw_text(
        &mut d,
        &grade_style.0,
        Align::Start,
        Align::Start,
        100,
        grade_style.1,
        Some((20, 20)),
        false,
        &ui_state
    );
    Align::draw_text(
        &mut d,
        &accuracy_txt,
        Align::Start,
        Align::Start,
        40,
        grade_style.1,
        Some((20, 110)),
        false,
        &ui_state
    );

    for (i, judgment) in judgments.iter().enumerate() {
        let amount = counts.get(judgment).unwrap_or(&0);
        let y_offset = 150 + (i * 20);

        Align::draw_text(
            &mut d,
            &format!("{}: {}", judgment.to_string(), amount),
            Align::Start,
            Align::Start,
            20,
            Color::WHITE,
            Some((20, y_offset as i32)),
            false,
            &ui_state
        );
    }

    Align::draw_text(
        &mut d,
        "Press space to go back...",
        Align::End,
        Align::Middle,
        20,
        Color::WHITE,
        Some((0, -10)),
        false,
        &ui_state
    );

    if d.is_key_pressed(KeyboardKey::KEY_SPACE) {
        *in_game_state = ProgramState::new(vec![], in_game_state.receptor_y);
        *song = None;
    }
}
