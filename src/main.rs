use raylib::prelude::*;
use serde_json::json;
mod game;
mod judgment;
mod models;
mod results;
use crate::{models::*, results::draw_results};

struct UIElements {
    pub fonts: Vec<Font>,
    pub lane_width: i32,
    pub note_height: i32,
    pub fg: Color,
    pub bg: Color,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let game_config: GameConfig = GameConfig::load();
    let viewport: Viewport = Viewport::new(0, 0, vec![], 0);
    let mut song_state: SongState = SongState::new();

    let mut current_error: String = String::new();

    let (mut rhl, rt) = raylib::init()
        .log_level(TraceLogLevel::LOG_NONE)
        .resizable()
        .height(480)
        .width(640)
        .msaa_4x()
        .build();

    rhl.set_window_min_size(640, 480);
    rhl.set_target_fps(game_config.max_fps);

    let audio_device: RaylibAudio = audio::RaylibAudio::init_audio_device()?;
    let mut song: Option<Music> = None;

    let main_font: Font = rhl.load_font_ex(&rt, "lt.ttf", 60, None)?;
    main_font.texture().set_texture_filter(&rt, TextureFilter::TEXTURE_FILTER_TRILINEAR);

    let ui: UIElements = UIElements {
        fonts: vec![main_font],
        lane_width: 100,
        note_height: 20,
        fg: Color::WHITE,
        bg: Color::BLACK,
    };

    let mut program_state: AppState = AppState::new(game_config, viewport, song_state, Screens::Menu, ui);

    let tap_sfx = audio_device.new_sound(&program_state.game_config.hitsound)?;
    while !rhl.window_should_close() {
        let mut d = rhl.begin_drawing(&rt);
        let sc_wdth_half = d.get_screen_width() / 2;
        program_state.viewport.w = d.get_screen_width();
        program_state.viewport.h = d.get_screen_height();
        d.clear_background(program_state.ui.bg);
        program_state.viewport.receptor_y = program_state.viewport.h - program_state.ui.note_height;
        match program_state.current_screen {
            Screens::Game => {
                let sg_clone = &program_state.song_state.song_data.clone();
                let num_lanes = sg_clone.clone().unwrap().lanes.clone();
                let lane_x_positions: Vec<(i32, KeyboardKey)> = (0..num_lanes)
                    .map(|i| {
                        let offset = (i as f32 - (num_lanes as f32 - 1.0) / 2.0) * program_state.ui.lane_width as f32;
                        let key = program_state.keys.get(i as usize).unwrap_or(&KeyboardKey::KEY_ZERO);
                        (sc_wdth_half + offset as i32, *key)
                    })
                    .collect();

                program_state.viewport.lanes = lane_x_positions;

                if d.is_key_pressed(KeyboardKey::KEY_F1) {
                    song_state = SongState::new();
                    song = None;
                }

                if let Some(song) = &mut song {
                    game::game_loop(
                        d,
                        &mut program_state,
                        song,
                        &tap_sfx
                    );
                }
            }
            Screens::Menu => {
                let mut draw_label = |text: &str, y_offset: i32| {
                    // ease the pain a little...
                    Align::draw_text(
                        &mut d,
                        text,
                        Align::Middle,
                        Align::Middle,
                        20,
                       program_state.ui.fg,
                        Some((0, y_offset)),
                        false,
                        &program_state.ui,
                    );
                };

                if let Some(ref s) = program_state.song_state.song_data {
                    draw_label(&s.name, -60);
                    draw_label(&format!("Notes: {}", s.notes.len()), -40);
                    draw_label(&s.difficulty_name, -20);

                    Align::draw_text(
                        &mut d,
                        "Hold shift to autoplay",
                        Align::End,
                        Align::Middle,
                        20,
                        program_state.ui.fg,
                        None,
                        false,
                        &program_state.ui,
                    );
                    Align::draw_text(
                        &mut d,
                        "Press space to begin...",
                        Align::Middle,
                        Align::Middle,
                        30,
                        program_state.ui.fg,
                        None,
                        false,
                        &program_state.ui,
                    );

                    if d.is_key_pressed(KeyboardKey::KEY_SPACE) {
                        program_state.song_state.timer = 0.;
                        program_state.song_state.song_timer = 0.;
                        program_state.current_screen = Screens::Game;
                        program_state.game_config.autoplay = d.is_key_down(KeyboardKey::KEY_LEFT_SHIFT);
                    }
                } else {
                    draw_label(&current_error, 20);

                    Align::draw_text(
                        &mut d,
                        "SUBYSMDSLAGC",
                        Align::Start,
                        Align::End,
                        30,
                        program_state.ui.fg,
                        Some((-20, 20)),
                        false,
                        &program_state.ui,
                    );
                    if d.is_file_dropped() {
                        let dropped_files = d.load_dropped_files();

                        if let Some(raw_path) = dropped_files.paths().get(0) {
                            match SongData::setup_map_and_get_song(raw_path.to_string(), &mut program_state.song_state) {
                                Ok(s) => song = Some(audio_device.new_music(s.to_str().unwrap())?),
                                Err(e) => current_error = e.to_string(),
                            };
                        }
                    }
                }
            }
            Screens::Results => {
                if let Some(ref song) = song {
                    song.update_stream();
                }
                if d.is_key_pressed(KeyboardKey::KEY_SPACE) {
                    program_state.song_state = SongState::new();
                    song = None;
                }
                draw_results(d, &mut program_state);
            }
            Screens::Songs => {}
        }
    }

    let whatever = json!({
        "scroll_speed":  program_state.game_config.scroll_speed,
        "visual_offset": program_state.game_config.visual_offset,
        "input_offset":  program_state.game_config.input_offset,
        "max_fps": program_state.game_config.max_fps,
        "hitsound": program_state.game_config.hitsound,
        "lane_1_key": program_state.game_config.lane_1_key,
        "lane_2_key": program_state.game_config.lane_2_key,
        "lane_3_key": program_state.game_config.lane_3_key,
        "lane_4_key": program_state.game_config.lane_4_key,
        "songs_path": program_state.game_config.songs_path
    });

    let str = whatever.to_string();
    std::fs::write("config.json", str)?;

    Ok(())
}
