use std::{fs, path::{Path, PathBuf}};

use rayon::iter::{IntoParallelIterator, ParallelIterator};
use serde::Deserialize;

use crate::{judgment::Judgment, models::{Note, SongData, SvPoint}};


#[derive(Debug, Deserialize)]
struct QuaFile {
    #[serde(rename = "Title")]
    title: String,
    #[serde(rename = "DifficultyName")]
    difficulty_name: String,
    #[serde(rename = "Mode")]
    mode: String,
    #[serde(rename = "AudioFile")]
    audio_file: String,
    #[serde(rename = "SliderVelocities")]
    slider_velocities: Vec<SliderVelocities>,
    #[serde(rename = "HitObjects")]
    hit_objects: Vec<QuaHitObject>,
}

#[derive(Debug, Deserialize, Clone)]
pub struct SliderVelocities {
    #[serde(rename = "StartTime")]
    #[serde(default)]
    start_time: f32,
    #[serde(rename = "Multiplier")]
    #[serde(default)]
    multiplier: Option<f32>,
}

#[derive(Debug, Deserialize)]
struct QuaHitObject {
    #[serde(rename = "StartTime", default)]
    start_time: f32,
    #[serde(rename = "Lane")]
    lane: usize,
    #[serde(rename = "EndTime", default)]
    end_time: f32,
}

pub fn load_qua_to_song_data(content: &str) -> Result<SongData, Box<dyn std::error::Error>> {
    let mut qua: QuaFile = serde_yaml_ng::from_str(content)?;

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
        })
        .filter(|obj: &Note| obj.lane <= 4)
        .collect();

    // let lanes: i32 = qua.mode.chars().filter(|c| c.is_digit(10)).collect::<String>().parse()?;
    // this gives a slight performance improvement
    let lanes: i32 = match qua.mode.as_str() {
        "Keys1" => 1,
        "Keys2" => 2,
        "Keys3" => 3,
        "Keys4" => 4,
        _ => {
            return Err("Error parsing Keys mode.".into());
        }
    };
    if lanes > 4 {
        return Err("Too many lanes! (max: 5)".into());
    }

    Ok(SongData {
        computed_sv: precompute_sv(&mut qua.slider_velocities),
        name: qua.title,
        song: qua.audio_file,
        difficulty_name: qua.difficulty_name,
        lanes,
        notes,
    })
}

pub fn parse_song_data(map_path: &PathBuf) -> Result<SongData, Box<dyn std::error::Error>> {
    if let Some(ext) = map_path.extension() {
        match fs::read_to_string(&map_path) {
            Ok(s) => {
                let mut song_data: SongData = match ext.to_str() {
                    Some("qua") => match load_qua_to_song_data(&s) {
                        Ok(song_data) => song_data,
                        Err(e) => return Err(format!("Quaver Error: {}", e).into()),
                    },
                    Some(&_) => return Err(format!("File format not recognized.").into()),
                    None => return Err(format!("Unable to load file due to an unknown reason.").into()),
                };

                let parent_dir = map_path.parent().unwrap_or(Path::new(".")).to_path_buf();
                song_data.song = parent_dir.join(&song_data.song).to_str().unwrap().to_string();
                return Ok(song_data);
            }
            Err(e) => return Err(format!("File Error: {}", e).into()),
        };
    } else {
        Err(format!("File has no extension!").into())
    }
}

pub fn precompute_sv(sv_list: &mut Vec<SliderVelocities>) -> Vec<SvPoint> {
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

pub fn load_charts<T: AsRef<Path>>(path: T) -> Result<Vec<SongData>, Box<dyn std::error::Error>> {
    let paths: Vec<_> = fs::read_dir(path)?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_dir())
        .flat_map(|e| fs::read_dir(e.path()).ok())
        .flatten()
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().map_or(false, |ext| ext == "qua"))
        .collect();
    let charts: Vec<SongData> = paths.into_par_iter().filter_map(|path| parse_song_data(&path).ok()).collect();
    Ok(charts)
}
