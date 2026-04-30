use crate::judgment::Judgment;
use raylib::{color::Color, ffi::KeyboardKey, input, text::Font};
use serde::Deserialize;

pub struct UIElements {
    pub fonts: Vec<Font>,
    pub text_scale: i32,
    pub lane_width: i32,
    pub note_height: i32,
    pub fg: Color,
    pub bg: Color,
    pub shadow: bool,
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
}

impl Note {
    pub fn is_missed(&self, current_time: f32) -> bool {
        if self.is_holding {
            return false;
        }

        let target_time = if self.end_time.is_some_and(|a| a != 0.) { self.end_time.unwrap_or(self.time) } else { self.time };

        self.state == Judgment::None && current_time > target_time + Judgment::Miss.threshold()
    }

    pub fn check_note_hit(notes: &mut [Note], lane: usize, current_time: f32) -> Option<f32> {
        if let Some(note) = notes
            .iter_mut()
            .find(|n| n.lane == lane && n.state == Judgment::None && (n.time - current_time).abs() <= Judgment::Miss.threshold())
        {
            note.accuracy = note.time - current_time;
            note.state = Judgment::from_time(note.accuracy.abs());

            return Some(note.accuracy);
        }
        None
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
    pub name: String,
    pub difficulty_name: String,
    pub lanes: i32,
    pub song: String,
    pub notes: Vec<Note>,
    pub computed_sv: Vec<SvPoint>,
    pub file_size: usize,
}

impl SongData {
    pub fn get_visual_time(&self, time: f32, sv: bool) -> f32 {
        let iidx = self.computed_sv.partition_point(|s| s.start_time <= time);
        if iidx == 0 {
            return time;
        }
        let point = &self.computed_sv[iidx - 1];
        if sv { point.visual_pos + (time - point.start_time) * point.multiplier } else { time }
    }
}

pub struct AppState {
    pub game_config: GameConfig,
    pub viewport: Viewport,
    pub song_state: SongState,
    pub song_modifiers: SongModifiers,
    pub current_screen: Screens,
    pub keys: Vec<KeyboardKey>,
    pub ui: UIElements,
}

impl AppState {
    pub fn new(viewport: Viewport, song_modifiers: SongModifiers, song_state: SongState, current_screen: Screens, ui: UIElements, game_config: GameConfig) -> Self {
        AppState {
            viewport,
            song_state,
            current_screen,
            keys: vec![
                input::key_from_i32(game_config.lane_1_key).unwrap_or(KeyboardKey::KEY_A),
                input::key_from_i32(game_config.lane_2_key).unwrap_or(KeyboardKey::KEY_S),
                input::key_from_i32(game_config.lane_3_key).unwrap_or(KeyboardKey::KEY_K),
                input::key_from_i32(game_config.lane_4_key).unwrap_or(KeyboardKey::KEY_L),
            ],
            ui,
            game_config,
            song_modifiers,
        }
    }
    pub fn init() -> Self {
        let game_config = GameConfig::load();

        let ui: UIElements = UIElements {
            fonts: vec![],
            lane_width: 100,
            note_height: 20,
            text_scale: 10,
            fg: Color::WHITE,
            bg: Color::BLACK,
            shadow: false,
        };
        AppState::new(
            Viewport::new(0, 0, vec![], 0),
            SongModifiers {
                autoplay: true,
                sv: true,
                no_anim: false,
                speed: 1.,
            },
            SongState::new(),
            Screens::Menu,
            ui,
            game_config,
        )
    }
}

#[derive(Debug, Deserialize)]
pub struct GameConfig {
    pub scroll_speed: f32,
    #[serde(default)]
    pub visual_offset: f32,
    #[serde(default)]
    pub input_offset: f32,
    pub max_fps: u32,
    #[serde(skip)]
    pub lane_1_key: i32,
    pub lane_2_key: i32,
    pub lane_3_key: i32,
    pub lane_4_key: i32,
    pub songs_path: String,
}

impl Default for GameConfig {
    fn default() -> Self {
        Self {
            scroll_speed: 20.,
            visual_offset: 0.,
            input_offset: 0.,
            max_fps: 60,
            lane_1_key: KeyboardKey::KEY_A as i32,
            lane_2_key: KeyboardKey::KEY_S as i32,
            lane_3_key: KeyboardKey::KEY_K as i32,
            lane_4_key: KeyboardKey::KEY_L as i32,
            songs_path: String::from("./charts/"),
        }
    }
}

impl GameConfig {
    pub fn load() -> Self {
        match std::fs::read_to_string("config.json") {
            Ok(content) => serde_json::from_str(&content).expect("Failed to parse config"),
            Err(error) => {
                println!("Issue loading configuration: {error}");
                GameConfig::default()
            }
        }
    }
}

pub struct SongModifiers {
    pub autoplay: bool,
    pub sv: bool,
    pub no_anim: bool,
    pub speed: f32,
}
pub enum Align {
    Start,
    Middle,
    End,
}

#[derive(Clone)]
pub struct Viewport {
    pub w: i32,
    pub h: i32,
    pub lanes: Vec<(i32, KeyboardKey)>,
    pub receptor_y: i32,
}

impl Viewport {
    pub fn new(w: i32, h: i32, l: Vec<(i32, KeyboardKey)>, r: i32) -> Viewport {
        Viewport { w, h, lanes: l, receptor_y: r }
    }
}

#[derive(PartialEq, Clone, Copy)]
pub enum Screens {
    Menu,
    Game,
    Results,
    Songs,
}

pub struct SongState {
    pub song_timer: f32,
    pub timer: f32,
    pub combo: i32,
    pub max_combo: i32,
    pub accuracy: f32,
    pub song_data: Option<SongData>,
}

impl SongState {
    pub fn new() -> SongState {
        SongState {
            song_timer: 0.0,
            timer: 0.0,
            combo: 0,
            accuracy: 0.,
            song_data: None,
            max_combo: 0,
        }
    }
}
