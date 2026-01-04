use std::cmp::Ordering;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct RawKey {
    time: f64,
    value: f64,
    #[serde(rename = "inTangent")]
    in_tangent: f64,
    #[serde(rename = "outTangent")]
    out_tangent: f64,
}

#[derive(Debug, Deserialize)]
struct RawCurve {
    keys: Vec<RawKey>,
    #[serde(rename = "preWrapMode")]
    pre_wrap_mode: Option<i32>,
    #[serde(rename = "postWrapMode")]
    post_wrap_mode: Option<i32>,
}

/// Single keyframe used by the curve evaluator
#[derive(Debug, Clone)]
pub struct Keyframe {
    pub time: f64,
    pub value: f64,
    pub in_tangent: f64,
    pub out_tangent: f64,
}

/// Simple AnimationCurve evaluator implementing cubic Hermite interpolation
#[derive(Debug, Clone)]
pub struct AnimationCurve {
    keys: Vec<Keyframe>,
    pre_wrap_mode: i32,
    post_wrap_mode: i32,
}

const WRAP_PINGPONG: i32 = 4;
const WRAP_CLAMPFOREVER: i32 = 8;

impl AnimationCurve {
    /// Load a curve from a JSON string matching the Unity export
    pub fn from_json(json: &str) -> Result<Self, serde_json::Error> {
        let raw: RawCurve = serde_json::from_str(json)?;
        let mut keys: Vec<Keyframe> = raw
            .keys
            .into_iter()
            .map(|k| Keyframe {
                time: k.time,
                value: k.value,
                in_tangent: k.in_tangent,
                out_tangent: k.out_tangent,
            })
            .collect();

        // Ensure keys are sorted by time
        keys.sort_by(|a, b| a.time.partial_cmp(&b.time).unwrap_or(Ordering::Equal));

        let pre = raw.pre_wrap_mode.unwrap_or(0);
        let post = raw.post_wrap_mode.unwrap_or(0);

        Ok(Self {
            keys,
            pre_wrap_mode: pre,
            post_wrap_mode: post,
        })
    }

    fn map_time(&self, t: f64) -> f64 {
        // If no keys or only a single key, nothing to map
        if self.keys.len() < 2 {
            return t;
        }

        let first = self.keys[0].time;
        let last = self.keys.last().unwrap().time;

        // If inside range nothing to do
        if t >= first && t <= last {
            return t;
        }

        let is_before = t < first;
        let mode = if is_before {
            self.pre_wrap_mode
        } else {
            self.post_wrap_mode
        };

        // ClampForever: clamp to boundary value
        if mode == WRAP_CLAMPFOREVER {
            return if is_before { first } else { last };
        }

        // PingPong: bounce back and forth within range
        if mode == WRAP_PINGPONG {
            let length = last - first;
            if length <= 0.0 {
                return first;
            }
            let relative = t - first;
            let period = 2.0 * length;
            // Use rem_euclid so negative times map correctly
            let m = relative.rem_euclid(period);
            // Map into a triangle wave: ping = length - |m - length|
            let ping = length - (m - length).abs();
            return first + ping;
        }

        // Default: clamp to boundary
        if is_before { first } else { last }
    }

    /// Evaluate the curve at `t` using cubic Hermite interpolation between adjacent keyframes.
    pub fn evaluate(&self, t: f64) -> f64 {
        if self.keys.is_empty() {
            return 0.0;
        }
        if self.keys.len() == 1 {
            return self.keys[0].value;
        }

        // map times outside key range according to wrap modes
        let mapped_t = self.map_time(t);

        if mapped_t <= self.keys[0].time {
            return self.keys[0].value;
        } else if mapped_t >= self.keys.last().unwrap().time {
            return self.keys.last().unwrap().value;
        }

        // find segment using mapped time
        let mut idx = 0usize;
        for i in 0..self.keys.len() - 1 {
            if mapped_t >= self.keys[i].time && mapped_t <= self.keys[i + 1].time {
                idx = i;
                break;
            }
        }

        let k0 = &self.keys[idx];
        let k1 = &self.keys[idx + 1];
        let dt = k1.time - k0.time;
        if dt <= 0.0 {
            return k0.value;
        }
        let x = (mapped_t - k0.time) / dt;

        // Hermite basis
        let t2 = x * x;
        let t3 = t2 * x;

        let h00 = 2.0 * t3 - 3.0 * t2 + 1.0;
        let h10 = t3 - 2.0 * t2 + x;
        let h01 = -2.0 * t3 + 3.0 * t2;
        let h11 = t3 - t2;

        let m0 = k0.out_tangent * dt;
        let m1 = k1.in_tangent * dt;

        h00 * k0.value + h10 * m0 + h01 * k1.value + h11 * m1
    }
}
