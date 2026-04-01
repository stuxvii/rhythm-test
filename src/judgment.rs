use serde::Deserialize;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Deserialize, Default)]
#[serde(rename_all = "PascalCase")]
pub enum Judgment {
    #[default]
    None,
    Flawless,
    Perfect,
    Nice,
    Okay,
    Ehhh,
    Miss,
}

impl Judgment {
    pub fn threshold(&self) -> f32 {
        match self {
            Self::Flawless => 0.020,
            Self::Perfect  => 0.035,
            Self::Nice     => 0.050,
            Self::Okay     => 0.150,
            Self::Ehhh     => 0.250,
            Self::Miss     => 0.500,
            Self::None     => f32::MAX,
        }
    }

    pub fn weight(&self) -> f32 {
        match self {
            Self::Flawless => 1.0,
            Self::Perfect  => 0.95,
            Self::Nice     => 0.9,
            Self::Okay     => 0.75,
            Self::Ehhh     => 0.25,
            Self::Miss     => 0.0,
            Self::None     => 0.0,
        }
    }

    pub fn from_time(time_diff: f32) -> Self {
        let diff = time_diff.abs();
        
        if diff <= Self::Flawless.threshold() { Self::Flawless }
        else if diff <= Self::Perfect.threshold() { Self::Perfect }
        else if diff <= Self::Nice.threshold()    { Self::Nice }
        else if diff <= Self::Okay.threshold()    { Self::Okay }
        else if diff <= Self::Ehhh.threshold()    { Self::Ehhh }
        else if diff >= Self::Miss.threshold()    { Self::Miss }
        else { Self::None }
    }
}

impl fmt::Display for Judgment {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match self {
            Self::Flawless => "FLAWLESS!!!",
            Self::Perfect  => "PERFECT!!",
            Self::Nice     => "NICE!",
            Self::Okay     => "OK",
            Self::Ehhh     => "EHHH",
            Self::Miss     => "MISS!",
            Self::None     => "",
        };
        write!(f, "{}", s)
    }
}