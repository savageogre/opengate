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
pub fn apply_global_fade(
    n: usize,
    total: usize,
    fade_len: usize,
    left: f32,
    right: f32,
) -> (f32, f32) {
    let g = if n < fade_len {
        n as f32 / fade_len as f32
    } else if n + fade_len >= total {
        let remain = total - n;
        (remain as f32 / fade_len as f32).clamp(0.0, 1.0)
    } else {
        1.0
    };
    (left * g, right * g)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Curve;

    fn approx_eq(a: f32, b: f32) -> bool {
        (a - b).abs() <= 1e-16
    }

    fn approx_eq_f64(a: f64, b: f64) -> bool {
        (a - b).abs() <= 1e-12
    }

    #[test]
    fn test_secs_to_samples() {
        assert_eq!(secs_to_samples(1.5, 48_000), 72_000);
        assert_eq!(secs_to_samples(0.5, 44_100), 22_050);
    }

    #[test]
    fn test_secs_to_samples_clamp_negative() {
        assert_eq!(secs_to_samples(-2.0, 48_000), 0);
    }

    #[test]
    fn test_secs_to_samples_rounding_behavior() {
        assert_eq!(secs_to_samples(0.4999, 48_000), 23_995);
        assert_eq!(secs_to_samples(0.5001, 48_000), 24_005);
    }

    #[test]
    fn test_ms_to_samples() {
        assert_eq!(ms_to_samples(50.0, 48_000), 2_400);
        assert_eq!(ms_to_samples(1000.0, 48_000), 48_000);
    }

    #[test]
    fn test_ms_to_samples_clamp_negative() {
        assert_eq!(ms_to_samples(-25.0, 48_000), 0);
    }

    #[test]
    fn test_lerp_f32() {
        assert!(approx_eq(lerp(0.0_f32, 10.0_f32, 0.0), 0.0));
        assert!(approx_eq(lerp(0.0_f32, 10.0_f32, 1.0), 10.0));
        assert!(approx_eq(lerp(0.0_f32, 10.0_f32, 0.5), 5.0));
        assert!(approx_eq(lerp(2.0_f32, 6.0_f32, 0.25), 3.0));
    }

    #[test]
    fn test_lerp_f64() {
        assert!(approx_eq_f64(lerp(0.0_f64, 1.0_f64, 0.25), 0.25));
        assert!(approx_eq_f64(lerp(2.0_f64, 6.0_f64, 0.25), 3.0));
    }

    #[test]
    fn test_ease_linear_identity_on_0_1() {
        assert!(approx_eq(ease(0.0, Curve::Linear), 0.0));
        assert!(approx_eq(ease(0.25, Curve::Linear), 0.25));
        assert!(approx_eq(ease(0.5, Curve::Linear), 0.5));
        assert!(approx_eq(ease(1.0, Curve::Linear), 1.0));
    }

    #[test]
    fn test_ease_linear_outside_0_1() {
        assert!(approx_eq(ease(-1.0, Curve::Linear), 0.0));
        assert!(approx_eq(ease(2.0, Curve::Linear), 1.0));
    }

    #[test]
    fn test_ease_exp_endpoints() {
        assert!(approx_eq(ease(0.0, Curve::Exp), 0.0));
        assert!(approx_eq(ease(1.0, Curve::Exp), 1.0));
    }

    #[test]
    fn test_ease_exp_monotonicity() {
        let a = ease(0.1, Curve::Exp);
        let b = ease(0.2, Curve::Exp);
        let c = ease(0.555, Curve::Exp);
        let d = ease(0.9, Curve::Exp);
        assert!(a < b && b < c && c < d);
    }

    #[test]
    fn test_ease_exp_below_linear_at_05_with_k4() {
        let mid = ease(0.5, Curve::Exp);
        assert!(mid > 0.0 && mid < 0.5);
    }

    #[test]
    fn test_apply_global_fade_start_middle_end() {
        let total = 1000usize;
        let fade_len = 100usize;

        // Start of file gain should be 0.0
        let (mut l, mut r) = (1.0_f32, -0.5_f32);
        (l, r) = apply_global_fade(0, total, fade_len, l, r);
        assert!(approx_eq(l, 0.0));
        assert!(approx_eq(r, 0.0));

        // Fade in region at 50, gain is 0.5
        let (mut l, mut r) = (1.0_f32, -0.5_f32);
        (l, r) = apply_global_fade(50, total, fade_len, l, r);
        assert!(approx_eq(l, 0.5));
        assert!(approx_eq(r, -0.25));

        // Middle should be unaffected...
        let (mut l, mut r) = (0.8_f32, -0.4_f32);
        (l, r) = apply_global_fade(500, total, fade_len, l, r);
        assert!(approx_eq(l, 0.8));
        assert!(approx_eq(r, -0.4));

        // At the last sample where n=99, remain is 1 so gain is 1/100
        let (mut l, mut r) = (1.0_f32, -0.5_f32);
        (l, r) = apply_global_fade(999, total, fade_len, l, r);
        assert!(approx_eq(l, 0.01));
        assert!(approx_eq(r, -0.005));

        // Inside the tail region where n=950, remainer is 50 so gain is 1/2
        let (mut l, mut r) = (1.0_f32, -0.5_f32);
        (l, r) = apply_global_fade(950, total, fade_len, l, r);
        assert!(approx_eq(l, 0.5));
        assert!(approx_eq(r, -0.25));
    }

    #[test]
    fn test_apply_global_fade_near_boundary_no_overlap_issue() {
        let total = 1000usize;
        let fade_len = 100usize;

        // Just after fade in ends
        let (mut l, mut r) = (1.0_f32, 1.0_f32);
        (l, r) = apply_global_fade(100, total, fade_len, l, r);
        assert!(approx_eq(l, 1.0));
        assert!(approx_eq(r, 1.0));

        // Just before fade out starts
        let (mut l, mut r) = (1.0_f32, 1.0_f32);
        (l, r) = apply_global_fade(899, total, fade_len, l, r);
        assert!(approx_eq(l, 1.0));
        assert!(approx_eq(r, 1.0));
    }
}
