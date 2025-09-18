use serde::Deserialize;
use std::path::PathBuf;

#[derive(Debug, Deserialize, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum Curve {
    Linear,
    Exp,
}

#[derive(Debug, Deserialize)]
pub struct Config {
    /// Output filename
    pub out: PathBuf,

    /// Optional overrides
    #[serde(default)]
    pub sample_rate: Option<u32>,
    #[serde(default)]
    pub gain: Option<f32>,
    #[serde(default)]
    pub fade_ms: Option<f32>,

    /// The sequence of audio segments
    pub segments: Vec<Segment>,
}

#[derive(Debug, Deserialize, Clone, Copy)]
pub struct ToneSpec {
    #[serde(default = "default_carrier")]
    pub carrier: f32,
    pub hz: f32,
}

/// Default carrier tone should be a reasonable 200.0 Hertz.
fn default_carrier() -> f32 {
    200.0
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum Segment {
    /// Keep a steady tone for the duration `dur`.
    Tone { dur: f32, carrier: f32, hz: f32 },
    /// Transition from -> to across duration, with an optional curve.
    Transition {
        dur: f32,
        from: ToneSpec,
        to: ToneSpec,
        #[serde(default)]
        curve: Option<Curve>,
    },
}

pub enum Chunk {
    Tone {
        samples: usize,
        spec: ToneSpec,
    },
    Transition {
        samples: usize,
        from: ToneSpec,
        to: ToneSpec,
        curve: Curve,
    },
}

impl Chunk {
    pub fn samples(&self) -> usize {
        match self {
            Chunk::Tone { samples, .. } => *samples,
            Chunk::Transition { samples, .. } => *samples,
        }
    }
}
