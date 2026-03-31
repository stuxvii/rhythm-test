use crate::judgement::Judgment;
use raylib::{
    color::Color,
    math::Vector2,
    prelude::{RaylibDraw, RaylibDrawHandle},
};
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone, Copy)]
pub struct Note {
    pub lane: usize,
    pub time: f32,
    pub end_time: Option<f32>,

    #[serde(default)]
    pub accuracy: f32,
    #[serde(default)]
    pub state: Judgment,
    #[serde(default)]
    pub is_holding: bool,
    #[serde(default)]
    pub empty: bool,
}

impl Note {
    pub fn is_missed(&self, current_time: f32) -> bool {
        if self.is_holding { return false; }

        let target_time = 
        if self.end_time.is_some_and(|a| a != 0.) {
            self.end_time.unwrap_or(self.time)
        } else {
            self.time
        };

        self.state == Judgment::None && current_time > target_time + Judgment::Miss.threshold()
    }

    pub fn check_note_hit(notes: &mut [Note], lane: usize, current_time: f32) -> Judgment {
        if let Some(note) = notes
            .iter_mut()
            .find(|n| n.lane == lane && n.state == Judgment::None && (n.time - current_time).abs() <= Judgment::Miss.threshold())
        {
            note.accuracy = note.time - current_time;
            note.state = Judgment::from_time(note.accuracy.abs());

            return note.state;
        }
        Judgment::None
    }

    pub fn accuracy(notes: &Vec<Note>) -> f32 {
        let judged_notes: Vec<&Note> = notes.iter().filter(|n| n.state != Judgment::None).collect();
        if judged_notes.is_empty() {
            return 100.0;
        }

        let total_weight: f32 = judged_notes.iter().map(|n| n.state.weight()).sum();
        (total_weight / judged_notes.len() as f32) * 100.0
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct SongData {
    pub bpm: f32,
    pub name: String,
    pub song: String,
    pub offset: f32,
    pub notes: Vec<Note>,
}

#[derive(Debug, Deserialize)]
pub struct GameConfig {
    pub scroll_speed: f32,
    pub max_fps: u32,
    pub hitsound: String,
    pub autoplay: bool,
    pub quit_after_song_end: bool,
    pub lane_1_key: i32, 
    pub lane_2_key: i32, 
    pub lane_3_key: i32, 
    pub lane_4_key: i32, 
}

impl GameConfig {
    pub fn load() -> Self {
        let content = std::fs::read_to_string("config.json").expect("Failed to read config");
        serde_json::from_str(&content).expect("Failed to parse config")
    }
}

pub enum Align {
    Start,
    Middle,
    End,
}

impl Align {
    pub fn draw_text(d: &mut RaylibDrawHandle, text: &str, vertical: Align, horizontal: Align, font_size: i32, color: Color, offset: Option<Vector2>) {
        let text_width = d.measure_text(text, font_size);
        let mut x = match horizontal {
            Align::Start => 0,
            Align::Middle => (d.get_screen_width() / 2) - (text_width / 2),
            Align::End => d.get_screen_width() - text_width,
        };
        let mut y = match vertical {
            Align::Start => 0,
            Align::Middle => (d.get_screen_height() / 2) - (font_size / 2),
            Align::End => d.get_screen_height() - font_size,
        };

        if let Some(v) = offset {
            x += v.x as i32;
            y += v.y as i32;
        }

        // d.draw_text_ex(font, text, Vector2::new(x as f32,y as f32), font_size as f32, 1., color);
        d.draw_text(text, x, y-2, font_size, color);
    }
}
