use rand::Rng;

/// Types of noise that can be layered with the beats
#[derive(Debug, Clone, Copy, serde::Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum NoiseColor {
    White,
    Pink,
    Brown,
}

#[derive(Debug, Clone)]
pub struct NoiseGenerator {
    color: NoiseColor,
    pink_state: [f32; 7],
    brown_last: f32,
}

impl NoiseGenerator {
    pub fn new(color: NoiseColor) -> Self {
        Self {
            color,
            pink_state: [0.0; 7],
            brown_last: 0.0,
        }
    }

    /// Generate the next noise sample (-1.0 .. 1.0)
    pub fn next_sample(&mut self) -> f32 {
        let mut rng = rand::rng();
        match self.color {
            NoiseColor::White => rng.random_range(-1.0..=1.0),

            NoiseColor::Pink => {
                // Paul Kellet's refined pink noise filter
                let white: f32 = rng.random_range(-1.0..=1.0);
                self.pink_state[0] = 0.99886 * self.pink_state[0] + white * 0.0555179;
                self.pink_state[1] = 0.99332 * self.pink_state[1] + white * 0.0750759;
                self.pink_state[2] = 0.96900 * self.pink_state[2] + white * 0.1538520;
                self.pink_state[3] = 0.86650 * self.pink_state[3] + white * 0.3104856;
                self.pink_state[4] = 0.55000 * self.pink_state[4] + white * 0.5329522;
                self.pink_state[5] = -0.7616 * self.pink_state[5] - white * 0.0168980;
                let out = self.pink_state.iter().take(6).sum::<f32>()
                    + self.pink_state[6]
                    + white * 0.5362;
                self.pink_state[6] = white * 0.115926;
                // Scale down
                out * 0.11
            }

            NoiseColor::Brown => {
                let white: f32 = rng.random_range(-1.0..=1.0);
                self.brown_last += white * 0.02;
                self.brown_last = self.brown_last.clamp(-1.0, 1.0);
                self.brown_last
            }
        }
    }
}
