/// Utilities and common math used by opengate.
use crate::config::Curve;
use num_traits::Float;

/// Given seconds and a sample rate, provide the number of samples.
#[inline]
pub fn secs_to_samples(secs: f32, sample_rate: u32) -> usize {
    ((secs.max(0.0)) * sample_rate as f32).round() as usize
}

/// Given milliseconds and a sample rate, provide the number of samples.
#[inline]
pub fn ms_to_samples(ms: f32, sample_rate: u32) -> usize {
    secs_to_samples(ms / 1000.0, sample_rate)
}

/// Common lerping function.
/// We expect f32 but it's templated regardless.
#[inline]
pub fn lerp<T: Float>(a: T, b: T, t: T) -> T {
    a + (b - a) * t
}

/// Common easing function that takes a type of curve and x at [0.0, 1.0]
pub fn ease(t: f32, curve: Curve) -> f32 {
    let x = t.clamp(0.0, 1.0);
    match curve {
        Curve::Linear => x,
        // f(x) = (e^(k*x) - 1) / (e^k - 1)
        // with k = 4.0 for a noticeable curve.
        Curve::Exp => {
            let k = 4.0_f32;
            ((k * x).exp() - 1.0) / (k.exp() - 1.0)
        }
    }
}

/// Apply a quick global fade-in and out to avoid clicks at file boundaries.
pub fn apply_global_fade(n: usize, total: usize, fade_len: usize, left: &mut f32, right: &mut f32) {
    if n < fade_len {
        let g = n as f32 / fade_len as f32;
        *left *= g;
        *right *= g;
    } else if n + fade_len >= total {
        let remain = total - n;
        let g = (remain as f32 / fade_len as f32).clamp(0.0, 1.0);
        *left *= g;
        *right *= g;
    }
}
