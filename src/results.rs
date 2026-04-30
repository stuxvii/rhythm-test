use raylib::prelude::*;
use std::collections::HashMap;

use crate::{
    design,
    judgment::{Judgment, Rating},
    models::{Align, AppState, Note},
};

pub fn draw_results(mut d: RaylibDrawHandle<'_>, app_state: &mut AppState) {
    if let Some(song_data) = &app_state.song_state.song_data {
        let total_accuracy = Note::accuracy(&song_data.notes).clamp(0., 100.);
        let accuracy_txt = if app_state.song_modifiers.autoplay {
            format!("AUTOPLAY")
        } else {
            format!("{:.2}%", total_accuracy)
        };
        let grade_style: (String, Color) = if app_state.song_modifiers.autoplay {
            (String::from("BOT"), Color::GRAY)
        } else {
            let sh = Rating::from_time(total_accuracy);
            (sh.display_info().0.into(), sh.display_info().1)
        };

        let mut counts = HashMap::new();
        for note in &song_data.notes {
            *counts.entry(note.state).or_insert(0) += 1;
        }

        let judgments = [Judgment::Marvelous, Judgment::Perfect, Judgment::Great, Judgment::Good, Judgment::Okay, Judgment::Miss];

        design::draw_text(&mut d, &grade_style.0, Align::Start, Align::Start, 100, grade_style.1, (20, 20), &app_state.ui);
        design::draw_text(&mut d, &accuracy_txt, Align::Start, Align::Start, 40, grade_style.1, (20, 110), &app_state.ui);

        for (i, j) in judgments.iter().enumerate() {
            let n = counts.get(j).unwrap_or(&0);
            let y_off = 150 + (i as i32 * 20);
            design::draw_text(&mut d, &format!("{j}: {n}"), Align::Start, Align::Start, 20, app_state.ui.fg, (20, y_off), &app_state.ui);
        }
    }
}
