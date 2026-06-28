use crate::judgment::Judgment;
use raylib::prelude::*;
use serde::Deserialize;

pub struct UIElements {
    pub fonts: Vec<Font>,
    pub text_scale: i32,
    pub lane_width: i32,
    pub note_height: i32,
    pub song_item_height: i32,
    pub fg: Color,
    pub bg: Color,
    pub shadow: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BpmPoint {
    pub start_time: f32, // in seconds
    pub bpm: f32,
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
pub struct ChartMetadata {
    pub name: String,
    pub song: String,
    pub banner: String,
    pub background: String,
    pub artist: String,
    pub creator: String,
    pub lanes: i32,
}

#[derive(Debug)]
pub struct SongData {
    pub name: String,
    pub banner: String,
    pub banner_texture: Option<Texture2D>,
    pub background: String,
    pub background_texture: Option<Texture2D>,
    pub difficulties: Vec<ChartData>,
}

#[derive(Debug, Clone)]
pub struct ChartData {
    pub name: String,
    pub metadata: ChartMetadata,
    pub notes: Vec<Note>,
    pub computed_sv: Vec<SvPoint>,
    pub bpm: Vec<BpmPoint>,
    pub hints: Vec<String>,
}

impl ChartData {
    pub fn get_visual_time(&self, time: f32, sv: bool) -> f32 {
        let iidx = self.computed_sv.partition_point(|s| s.start_time <= time);
        if iidx == 0 {
            return time;
        }
        let point = &self.computed_sv[iidx - 1];
        if sv { point.visual_pos + (time - point.start_time) * point.multiplier } else { time }
    }
    pub fn get_bpm(&self, time: f32) -> f32 {
        
        let iidx = self.bpm.partition_point(|s| s.start_time <= time);
        if iidx == 0 {
            return 120.; // safe value?
        }
        let point = &self.bpm[iidx - 1];
        point.bpm
    }
}

pub struct AppState {
    pub config: GameConfig,
    pub viewport: Viewport,
    pub song_state: PlayState,
    pub song_modifiers: SongModifiers,
    pub current_screen: Screens,
    pub keys: Vec<KeyboardKey>,
    pub ui: UIElements,
}

impl AppState {
    pub fn new(viewport: Viewport, song_modifiers: SongModifiers, song_state: PlayState, current_screen: Screens, ui: UIElements, config: GameConfig) -> Self {
        AppState {
            viewport,
            song_state,
            current_screen,
            keys: vec![config.keybinds.left, config.keybinds.down, config.keybinds.up, config.keybinds.right],
            ui,
            config,
            song_modifiers,
        }
    }
    pub fn init() -> Result<AppState, Box<dyn std::error::Error>> {
        let game_config = GameConfig::load();

        let ui: UIElements = UIElements {
            fonts: vec![],
            lane_width: 100,
            note_height: 20,
            text_scale: 10,
            song_item_height: 60,
            fg: Color::WHITE,
            bg: Color::BLACK,
            shadow: true,
        };
        let result = AppState::new(
            Viewport::new(0, 0, vec![], 0),
            SongModifiers {
                autoplay: false,
                sv: true,
                speed: 1.,
            },
            PlayState::new(),
            Screens::StartMenu,
            ui,
            game_config?,
        );

        Ok(result)
    }

    pub fn json(&self) -> serde_json::Value {
        serde_json::json!({
            "scroll_speed": self.config.scroll_speed,
            "visual_offset":self.config.visual_offset,
            "input_offset": self.config.input_offset,
            "max_fps":      self.config.max_fps,
            "left":   self.config.keybinds.left as i32,
            "down":   self.config.keybinds.down as i32,
            "up":   self.config.keybinds.up as i32,
            "right":   self.config.keybinds.right as i32,
            "confirm":   self.config.keybinds.confirm as i32,
            "back":   self.config.keybinds.back as i32,
            "songs_path":   self.config.songs_path,
            "load_images":   self.config.load_images,
            "starry_field":   self.config.starry_field,
        })
    }
}

#[derive(Debug)]
pub struct Keybinds {
    pub left: KeyboardKey,
    pub down: KeyboardKey,
    pub up: KeyboardKey,
    pub right: KeyboardKey,
    pub confirm: KeyboardKey,
    pub back: KeyboardKey,
}
impl Default for Keybinds {
    fn default() -> Self {
        Self {
            left: KeyboardKey::KEY_D,
            down: KeyboardKey::KEY_F,
            up: KeyboardKey::KEY_J,
            right: KeyboardKey::KEY_K,
            confirm: KeyboardKey::KEY_Z,
            back: KeyboardKey::KEY_X,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct GameConfig {
    pub scroll_speed: f32,
    #[serde(default)]
    pub visual_offset: f32,
    #[serde(default)]
    pub input_offset: f32,
    #[serde(default)]
    pub max_fps: i32,
    #[serde(default = "default_true")]
    pub load_images: bool,
    #[serde(default = "default_true")]
    pub starry_field: bool,
    #[serde(skip)]
    pub keybinds: Keybinds,
    pub songs_path: String,
}
const fn default_true() -> bool {
    true
}
impl Default for GameConfig {
    fn default() -> Self {
        Self {
            scroll_speed: 20.,
            visual_offset: 0.,
            input_offset: 0.,
            max_fps: 60,
            load_images: false,
            starry_field: false,
            keybinds: Keybinds::default(),
            songs_path: String::from("./charts/"),
        }
    }
}

impl GameConfig {
    pub fn load() -> Result<GameConfig, Box<dyn std::error::Error>> {
        let content = match std::fs::read_to_string("config.json") {
            Ok(content) => content,
            Err(error) => {
                println!("Issue reading configuration: {error}");
                return Ok(GameConfig::default());
            }
        };

        let config = serde_json::from_str(&content).map_err(|e| format!("Issue parsing config JSON: {e}"))?;

        Ok(config)
    }
}

pub struct SongModifiers {
    pub autoplay: bool,
    pub sv: bool,
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
    StartMenu,
    Game,
    Results,
    Songs,
    Settings,
}

pub struct PlayState {
    pub song_timer: f32,
    pub timer: f32,
    pub combo: i32,
    pub max_combo: i32,
    pub accuracy: f32,
    pub song_data: Option<ChartData>,
}

impl PlayState {
    pub fn new() -> PlayState {
        PlayState {
            song_timer: 0.0,
            timer: 0.0,
            combo: 0,
            accuracy: 0.,
            song_data: None,
            max_combo: 0,
        }
    }
}

#[inline]
pub fn rrect<T1: AsF32, T2: AsF32, T3: AsF32, T4: AsF32>(x: T1, y: T2, width: T3, height: T4) -> Rectangle {
    Rectangle::new(x.as_f32(), y.as_f32(), width.as_f32(), height.as_f32())
}
