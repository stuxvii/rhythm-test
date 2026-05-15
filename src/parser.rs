use std::{
    fs,
    path::{Path, PathBuf},
    time::SystemTime,
};

use raylib::{RaylibThread, prelude::RaylibDrawHandle, texture::Texture2D};
use rayon::{
    iter::{IntoParallelIterator, ParallelBridge, ParallelIterator},
    slice::ParallelSliceMut,
    vec,
};
use serde::Deserialize;

use crate::{
    judgment::Judgment,
    models::{ChartData, ChartMetadata, Note, SongData, SvPoint},
};

pub trait Chart {
    fn parse(content: &str) -> Result<ChartData, Box<dyn std::error::Error>>;
}

#[derive(Debug, Deserialize)]
struct QuaFile {
    #[serde(rename = "Title")]
    title: String,
    #[serde(default)]
    #[serde(rename = "Creator")]
    creator: String,
    #[serde(default)]
    #[serde(rename = "Artist")]
    artist: String,
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
    #[serde(rename = "BannerFile")]
    #[serde(default)]
    banner_file: String,
}

impl Chart for QuaFile {
    fn parse(content: &str) -> Result<ChartData, Box<dyn std::error::Error>> {
        let mut qua: QuaFile = serde_yaml_ng::from_str(content)?;

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

        let notes: Vec<Note> = qua
            .hit_objects
            .into_par_iter()
            .filter(|obj| obj.lane <= lanes as usize)
            .map(|obj: QuaHitObject| Note {
                lane: obj.lane,
                time: obj.start_time / 1000.0,
                end_time: (obj.end_time > 0.0).then(|| obj.end_time / 1000.0),

                accuracy: 0.0,
                state: Judgment::None,
                is_holding: false,
            })
            .collect();


        let hints = vec![
            format!("artist: {}", qua.artist),
            format!("creator: {}", qua.creator),
            format!("notes: {}", notes.len()),
        ];

        Ok(ChartData {
            name: qua.title,
            metadata: ChartMetadata {
                name: qua.difficulty_name,
                song: qua.audio_file,
                banner: qua.banner_file,
                lanes,
                artist: qua.artist,
                creator: qua.creator,
            },
            computed_sv: precompute_sv(&mut qua.slider_velocities),
            notes,
            hints
        })
    }
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

pub fn parse_song_data(map_path: &PathBuf) -> Result<ChartData, Box<dyn std::error::Error>> {
    if let Some(ext) = map_path.extension() {
        match fs::read_to_string(&map_path) {
            Ok(content) => {
                let mut song_data: ChartData = match ext.to_str() {
                    Some("qua") => match QuaFile::parse(&content) {
                        Ok(song_data) => song_data,
                        Err(e) => return Err(format!("Quaver Error: {}", e).into()),
                    },
                    Some(&_) => return Err(format!("File format not recognized.").into()),
                    None => return Err(format!("Unable to load file due to an unknown reason.").into()),
                };

                let parent_dir = map_path.parent().unwrap_or(Path::new(".")).to_path_buf();
                song_data.metadata.song = parent_dir.join(&song_data.metadata.song).to_str().unwrap().to_string();
                if !song_data.metadata.banner.is_empty() {
                    song_data.metadata.banner = parent_dir.join(&song_data.metadata.banner).to_str().unwrap().to_string();
                }
                return Ok(song_data);
            }
            Err(e) => return Err(format!("File Error: {}", e).into()),
        };
    } else {
        Err(format!("File has no extension!").into())
    }
}

pub fn precompute_sv(sv_list: &mut Vec<SliderVelocities>) -> Vec<SvPoint> {
    sv_list.par_sort_unstable_by(|a, b| a.start_time.partial_cmp(&b.start_time).unwrap());

    let mut computed = Vec::with_capacity(sv_list.len());

    let mut last_visual_pos = 0.0;
    let mut last_time = 0.0;
    let mut last_mult = 1.0;

    for sv in sv_list {
        let start_time_secs = sv.start_time / 1000.0;
        let time_passed = start_time_secs - last_time;

        last_visual_pos += time_passed * last_mult;

        let current_mult = sv.multiplier.unwrap_or(1.0);

        computed.push(SvPoint {
            start_time: start_time_secs,
            multiplier: current_mult,
            visual_pos: last_visual_pos,
        });

        last_time = start_time_secs;
        last_mult = current_mult;
    }
    computed
}

/**
 * if someone can help out and optimize the chart parsing in this (main bottleneck) then that would be hella appreciated
 */
pub fn load_charts<T: AsRef<Path>>(path: T) -> Result<Vec<SongData>, Box<dyn std::error::Error>> {
    let songs: Vec<SongData> = fs::read_dir(path)?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().is_dir())
        .par_bridge()
        .filter_map(|e| {
            let files = fs::read_dir(e.path()).ok()?;
            let difficulties: Vec<ChartData> = files
                .flatten()
                .map(|entry| entry.path())
                .filter(|path| path.extension().map_or(false, |ext| ext == "qua"))
                .filter_map(|path| parse_song_data(&path).ok())
                .collect();

            if difficulties.is_empty() {
                return None;
            }

            let name = difficulties.first()?.name.clone();
            let banner = difficulties.first()?.metadata.banner.clone();

            Some(SongData { name, banner, banner_texture: None, difficulties })
        })
        .collect();
    Ok(songs)
}
