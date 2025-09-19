use regex::Regex;
use std::str::FromStr;
use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DurationSeconds(pub f32);

impl From<DurationSeconds> for Duration {
    fn from(ds: DurationSeconds) -> Self {
        Duration::from_secs_f32(ds.0)
    }
}

impl FromStr for DurationSeconds {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Try parsing a plain number first, with no suffix like h/m/s.
        if let Ok(val) = s.parse::<f32>() {
            return Ok(DurationSeconds(val));
        }

        // Regex for [h][m][s], each optional, with int/float
        // Less than zero MUST have 0 preceding, eg: 0.5m not .5m
        let re = Regex::new(r"^(?:(\d+(?:\.\d+)?)h)?(?:(\d+(?:\.\d+)?)m)?(?:(\d+(?:\.\d+)?)s)?$")
            .unwrap();

        if let Some(caps) = re.captures(s) {
            let mut secs = 0.0;

            if let Some(h) = caps.get(1) {
                secs += h
                    .as_str()
                    .parse::<f32>()
                    .map_err(|_| format!("Invalid hours: {:?}", h))?
                    * 3600.0;
            }
            if let Some(m) = caps.get(2) {
                secs += m
                    .as_str()
                    .parse::<f32>()
                    .map_err(|_| format!("Invalid minutes: {:?}", m))?
                    * 60.0;
            }
            if let Some(s) = caps.get(3) {
                secs += s
                    .as_str()
                    .parse::<f32>()
                    .map_err(|_| format!("Invalid seconds: {:?}", s))?;
            }

            if secs > 0.0 {
                return Ok(DurationSeconds(secs));
            } else {
                return Err(format!(
                    "Invalid seconds amount {:?} in format {:?}",
                    secs, s
                ));
            }
        }

        Err(format!("Invalid duration format: {:?}", s))
    }
}

use serde::{Deserialize, Deserializer};

impl<'de> Deserialize<'de> for DurationSeconds {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = String::deserialize(deserializer)?;
        raw.parse::<DurationSeconds>()
            .map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_secs(input: &str, expected: f32) {
        let dur: DurationSeconds = input.parse().unwrap();
        assert!(
            (dur.0 - expected).abs() < f32::EPSILON,
            "expected {} got {}",
            expected,
            dur.0
        );
    }

    #[test]
    fn test_plain_numbers() {
        assert_secs("30", 30.0);
        assert_secs("30.0", 30.0);
        assert_secs("45.1", 45.1);
    }

    #[test]
    fn test_seconds() {
        assert_secs("30s", 30.0);
        assert_secs("30.5s", 30.5);
    }

    #[test]
    fn test_minutes() {
        assert_secs("0.5m", 30.0);
        assert_secs("60m", 3600.0);
        assert_secs("1m30s", 90.0);
        assert_secs("0.5m10.5s", 40.5);
    }

    #[test]
    fn test_hours() {
        assert_secs("1h", 3600.0);
        assert_secs("3h30m", 3.0 * 3600.0 + 30.0 * 60.0);
        assert_secs("0.5h", 1800.0);
        assert_secs("1h2m3s", 3723.0);
    }

    #[test]
    fn test_invalid() {
        assert!("abc".parse::<DurationSeconds>().is_err());
        assert!("".parse::<DurationSeconds>().is_err());
        assert!("1x".parse::<DurationSeconds>().is_err());
        assert!("1d".parse::<DurationSeconds>().is_err());
        assert!(".5h".parse::<DurationSeconds>().is_err());
        assert!(".5m".parse::<DurationSeconds>().is_err());
        assert!(".5s".parse::<DurationSeconds>().is_err());
    }
}
