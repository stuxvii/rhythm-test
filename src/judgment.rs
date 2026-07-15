use raylib::color::Color;
use serde::Deserialize;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Deserialize, Default)]
#[serde(rename_all = "PascalCase")]
pub enum Judgment {
    #[default]
    None,
    Marvelous,
    Perfect,
    Great,
    Good,
    Okay,
    Miss,
}

impl Judgment {
    pub fn threshold(&self) -> f32 {
        match self {
            Self::Marvelous => 0.023,
            Self::Perfect => 0.057,
            Self::Great => 0.101,
            Self::Good => 0.141,
            Self::Okay => 0.169,
            Self::Miss => 0.218,
            Self::None => f32::MAX,
        }
    }

    pub fn weight(&self) -> f32 {
        match self {
            Self::Marvelous => 1.0,
            Self::Perfect => 0.95,
            Self::Great => 0.9,
            Self::Good => 0.75,
            Self::Okay => 0.25,
            Self::Miss => 0.0,
            Self::None => 0.0,
        }
    }

    pub fn from_time(time_diff: f32) -> Self {
        let diff = time_diff.abs();

        if diff <= Self::Marvelous.threshold() {
            Self::Marvelous
        } else if diff <= Self::Perfect.threshold() {
            Self::Perfect
        } else if diff <= Self::Great.threshold() {
            Self::Great
        } else if diff <= Self::Good.threshold() {
            Self::Good
        } else if diff <= Self::Okay.threshold() {
            Self::Okay
        } else if diff >= Self::Miss.threshold() {
            Self::Miss
        } else {
            Self::None
        }
    }

    pub fn color(&self) -> Color {
        match self {
            Self::Marvelous => Color::LIGHTGOLDENRODYELLOW,
            Self::Perfect => Color::GOLD,
            Self::Great => Color::GREEN,
            Self::Good => Color::BLUE,
            Self::Okay => Color::PINK,
            Self::Miss => Color::RED,
            Self::None => Color::BLANK,
        }
    }

    pub fn short_form(&self) -> &str {
        match self {
            Self::Marvelous => "MV",
            Self::Perfect => "PF",
            Self::Great => "GR",
            Self::Good => "GD",
            Self::Okay => "OK",
            Self::Miss => "MS",
            Self::None => "",
        }
    }
}

impl fmt::Display for Judgment {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let s = match self {
            Self::Marvelous => "Marvelous",
            Self::Perfect => "Perfect",
            Self::Great => "Great",
            Self::Good => "Good",
            Self::Okay => "Okay",
            Self::Miss => "Miss",
            Self::None => "",
        };
        write!(f, "{}", s)
    }
}

pub enum Rating {
    X,
    SP,
    S,
    A,
    B,
    C,
    D,
    F,
}

impl Rating {
    pub fn display_info(&self) -> (&str, raylib::prelude::Color) {
        let string = match self {
            Self::X => "X",
            Self::SP => "S+",
            Self::S => "S",
            Self::A => "A",
            Self::B => "B",
            Self::C => "C",
            Self::D => "D",
            Self::F => "F",
        };
        let color = match self {
            Self::X => Color::WHITE,
            Self::SP => Color::LIGHTGOLDENRODYELLOW,
            Self::S => Color::GOLD,
            Self::A => Color::GREEN,
            Self::B => Color::BLUE,
            Self::C => Color::PINK,
            Self::D => Color::RED,
            Self::F => Color::DARKRED,
        };

        (string, color)
    }
    pub fn threshold(&self) -> f32 {
        match self {
            Self::X => 100.,
            Self::SP => 99.,
            Self::S => 95.,
            Self::A => 90.,
            Self::B => 80.,
            Self::C => 70.,
            Self::D => 60.,
            Self::F => 0.,
        }
    }

    pub fn from_time(percentage: f32) -> Self {
        if percentage >= Self::X.threshold() {
            Self::X
        } else if percentage >= Self::SP.threshold() {
            Self::SP
        } else if percentage >= Self::S.threshold() {
            Self::S
        } else if percentage >= Self::A.threshold() {
            Self::A
        } else if percentage >= Self::B.threshold() {
            Self::B
        } else if percentage >= Self::C.threshold() {
            Self::C
        } else if percentage >= Self::D.threshold() {
            Self::D
        } else if percentage >= Self::F.threshold() {
            Self::F
        } else {
            Self::F
        }
    }
}
