use std::time;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScaleUnit {
    Pounds,
    Ounces,
    Grams,
    Kilograms
}

impl fmt::Display for ScaleUnit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ScaleUnit::Pounds => write!(f, "lb"),
            ScaleUnit::Ounces => write!(f, "oz"),
            ScaleUnit::Grams => write!(f, "g"),
            ScaleUnit::Kilograms => write!(f, "kg"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ScaleWeight {
    pub unit: ScaleUnit,
    pub value: f32,
    pub stable: bool,
    pub time: time::SystemTime
}

#[derive(Debug)]
pub enum WeightConversionError {
    InvalidString,
    InvalidStable,
    InvalidWeight,
    InvalidUnit
}

impl ScaleWeight {
    pub fn from_str(s: &str) -> Result<Self, WeightConversionError> {
        if s.len() < 16 {
            return Err(WeightConversionError::InvalidString)
        }

        let stable = match &s[..2] {
            "ST" => {
                true
            },
            "US" => {
                false
            },
            _ => {
                return Err(WeightConversionError::InvalidStable)
            }
        };

        let value_str = s[6..14].trim();
        let value: f32 = match value_str.parse() {
            Ok(v) => v,
            Err(_) => {
                return Err(WeightConversionError::InvalidWeight)
            }
        };

        let unit = match s[15..].trim() {
            "oz" => {
                ScaleUnit::Ounces
            },
            "lb" => {
                ScaleUnit::Pounds
            },
            "g" => {
                ScaleUnit::Grams
            },
            "kg" => {
                ScaleUnit::Kilograms
            }
            _ => {
                return Err(WeightConversionError::InvalidUnit)
            }
        };

        Ok(Self {
            unit,
            value,
            stable,
            time: time::SystemTime::now()
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ScaleStatus {
    OpenSucceeded(String),
    OpenFailed(String),
    Weight(ScaleWeight),
    Disconnected
}

#[derive(Debug, Clone, PartialEq)]
pub enum SerialCommand {
    OpenPort(String),
    Start,
    Stop,
}
