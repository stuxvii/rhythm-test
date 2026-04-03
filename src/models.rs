use crate::judgment::Judgment;
use raylib::{
    color::Color,
    ffi::KeyboardKey,
    math::Vector2,
    prelude::{RaylibDraw, RaylibDrawHandle},
};
use serde::Deserialize;

use std::path::Path;
use std::{fs, path::PathBuf};

#[derive(Debug, Deserialize)]
struct QuaFile {
    #[serde(rename = "Title")]
    title: String,
    #[serde(rename = "AudioFile")]
    audio_file: String,
    #[serde(rename = "TimingPoints")]
    timing_points: Vec<TimingPoint>,
    #[serde(rename = "SliderVelocities")]
    slider_velocities: Vec<SliderVelocities>,
    #[serde(rename = "HitObjects")]
    hit_objects: Vec<QuaHitObject>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct SliderVelocities {
    #[serde(rename = "StartTime")]
    start_time: f32,
    #[serde(rename = "Multiplier")]
    multiplier: Option<f32>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct TimingPoint {
    #[serde(rename = "StartTime")]
    start_time: Option<f32>,
    #[serde(rename = "Bpm")]
    bpm: f32,
}

#[derive(Debug, Deserialize)]
struct QuaHitObject {
    #[serde(rename = "StartTime")]
    start_time: f32,
    #[serde(rename = "Lane")]
    lane: usize,
    #[serde(rename = "EndTime", default)]
    end_time: f32,
}

#[derive(Debug, Clone)]
pub struct SvPoint {
    pub start_time: f32, // in seconds
    pub multiplier: f32,
    pub visual_pos: f32, // cumulative visual time
}

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
        if self.is_holding {
            return false;
        }

        let target_time = if self.end_time.is_some_and(|a| a != 0.) {
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
        let judged_notes = notes.iter().filter(|n| n.state != Judgment::None);

        let count = judged_notes.clone().count();
        if count == 0 {
            return 100.0;
        }

        let total_weight: f32 = judged_notes.map(|n| n.state.weight()).sum();
        (total_weight / count as f32) * 100.0
    }
}

#[derive(Debug, Clone)]
pub struct SongData {
    bpm: Vec<TimingPoint>,
    sv: Vec<SliderVelocities>,
    pub name: String,
    pub song: String,
    pub offset: f32,
    pub notes: Vec<Note>,
    pub computed_sv: Vec<SvPoint>,
}

impl SongData {
    pub fn load_qua_to_song_data(content: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let qua: QuaFile = serde_yaml::from_str(content)?;

        let notes = qua
            .hit_objects
            .into_iter()
            .map(|obj: QuaHitObject| Note {
                lane: obj.lane,
                time: obj.start_time / 1000.0,
                end_time: if obj.end_time > 0.0 { Some(obj.end_time / 1000.0) } else { None },

                accuracy: 0.0,
                state: Judgment::None,
                is_holding: false,
                empty: false,
            })
            .collect();

        Ok(Self {
            computed_sv: SongData::precompute_sv(qua.slider_velocities.clone()),
            sv: qua.slider_velocities,
            bpm: qua.timing_points,
            name: qua.title,
            song: qua.audio_file,
            offset: 0.0,
            notes,
        })
    }

    pub fn setup_map_and_get_song(raw_path: String, in_game_state: &mut ProgramState) -> Result<PathBuf, Box<dyn std::error::Error>> {
        let map_path = PathBuf::from(raw_path);
        let mut song_path: PathBuf = PathBuf::new();
        match fs::read_to_string(&map_path) {
            Ok(s) => {
                if let Some(ext) = map_path.extension() {
                    let song_data: SongData = match ext.to_str() {
                        Some("qua") => match SongData::load_qua_to_song_data(&s) {
                            Ok(song_data) => song_data,
                            Err(e) => return Err(format!("Quaver Error: {}", e).into()),
                        },
                        Some(&_) => return Err(format!("File format not recognized.").into()),
                        None => return Err(format!("Unable to load file due to an unknown reason.").into()),
                    };

                    let parent_dir = map_path.parent().unwrap_or(Path::new(".")).to_path_buf();
                    song_path = parent_dir.join(&song_data.song);
                    in_game_state.notes_to_draw = song_data.notes.iter().filter(|n| !n.empty).cloned().collect();
                    in_game_state.song_data = Some(song_data);
                }
            }
            Err(e) => return Err(format!("File Error: {}", e).into()),
        };
        Ok(song_path)
    }

    pub fn precompute_sv(mut sv_list: Vec<SliderVelocities>) -> Vec<SvPoint> {
        sv_list.sort_by(|a, b| a.start_time.partial_cmp(&b.start_time).unwrap());

        let mut computed = Vec::new();
        let mut last_visual_pos = 0.0;
        let mut last_time = 0.0;
        let mut last_mult = 1.0;

        for sv in sv_list {
            let start_time_secs = sv.start_time / 1000.0;
            let time_passed = start_time_secs - last_time;

            last_visual_pos += time_passed * last_mult;

            computed.push(SvPoint {
                start_time: start_time_secs,
                multiplier: sv.multiplier.unwrap_or(1.),
                visual_pos: last_visual_pos,
            });

            last_time = start_time_secs;
            last_mult = sv.multiplier.unwrap_or(1.0);
        }
        computed
    }

    pub fn bpm(&self, current_time: f32) -> f32 {
        let x = self.bpm.iter().filter(|a| a.start_time < Some(current_time)).last();
        if let Some(tp) = x { tp.bpm } else { 120. }
    }

    pub fn get_visual_time(&self, time: f32) -> f32 {
        let iidx = self.computed_sv.partition_point(|s| s.start_time <= time);
        if iidx == 0 {
            return time;
        }
        let point = &self.computed_sv[iidx - 1];
        point.visual_pos + (time - point.start_time) * point.multiplier
    }
}

#[derive(Debug, Deserialize)]
pub struct GameConfig {
    pub scroll_speed: f32,
    pub max_fps: u32,
    pub hitsound: String,
    #[serde(skip)]
    pub autoplay: bool,
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
    pub fn draw_text(
        d: &mut RaylibDrawHandle,
        text: &str,
        vertical: Align,
        horizontal: Align,
        font_size: i32,
        color: Color,
        offset: Option<Vector2>,
        shadow: bool,
    ) {
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

        if shadow {
            let opposite_color = Color::new(255 - color.r, 255 - color.g, 255 - color.b, 255);
            d.draw_text(text, x+1, y - 1, font_size, opposite_color);
            d.draw_text(text, x+1, y + 3, font_size, opposite_color);
            d.draw_text(text, x-1, y + 1, font_size, opposite_color);
            d.draw_text(text, x-1, y - 3, font_size, opposite_color);
        }
        d.draw_text(text, x, y -2, font_size, color);
    }
}

#[derive(Clone, Copy)]
pub struct ScreenDimension {
    pub w: i32,
    pub h: i32,
}

impl ScreenDimension {
    pub fn new(w: i32, h: i32) -> ScreenDimension {
        ScreenDimension { w, h }
    }
}

#[derive(PartialEq, Clone, Copy)]
pub enum Screens {
    Menu,
    Game,
    Results,
}

pub struct ProgramState {
    pub lanes: [(i32, KeyboardKey); 4],
    pub receptor_y: i32,
    pub current_song_timer: f32,
    pub current_timer: f32,
    pub notes_to_draw: Vec<Note>,
    pub combo: i32,
    pub cur_judge: Judgment,
    pub current_screen: Screens,
    pub song_data: Option<SongData>,
}

impl ProgramState {
    pub fn new(l: [(i32, KeyboardKey); 4], r: i32) -> ProgramState {
        ProgramState {
            current_song_timer: 0.0,
            current_timer: 0.0,
            lanes: l,
            receptor_y: r,
            notes_to_draw: vec![],
            combo: 0,
            cur_judge: Judgment::None,
            current_screen: Screens::Menu,
            song_data: None,
        }
    }
}
