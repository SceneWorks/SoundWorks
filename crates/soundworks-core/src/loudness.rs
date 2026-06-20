//! ITU-R BS.1770-4 gated integrated loudness (LUFS) and ≥4× oversampled
//! true-peak (dBTP).
//!
//! Replaces the previous plain-RMS-dBFS / sample-peak approximations (F-022)
//! that were mislabeled as LUFS / true peak. Dependency-free: K-weighting is two
//! cascaded biquads whose coefficients are computed for the actual sample rate
//! (the libebur128 method), gating follows BS.1770-4 (−70 LUFS absolute + −10 LU
//! relative gate over 400 ms / 75%-overlap blocks), and true peak uses a 4×
//! windowed-sinc polyphase interpolator.

use std::f64::consts::PI;

/// Loudness measurement for an audio buffer.
pub(crate) struct LoudnessStats {
    pub loudness_lufs: f32,
    pub true_peak_dbfs: f32,
}

/// Returned when the signal is silent or too short to gate, so the serialized
/// metadata stays finite rather than `-inf`/`NaN`.
const SILENCE_FLOOR_LUFS: f32 = -120.0;

#[derive(Clone, Copy)]
struct Biquad {
    b0: f64,
    b1: f64,
    b2: f64,
    a1: f64,
    a2: f64,
}

impl Biquad {
    /// BS.1770 stage 1: high-shelf "pre-filter" (libebur128 coefficients,
    /// recomputed for `fs` via the bilinear transform).
    fn k_weighting_shelf(fs: f64) -> Self {
        let f0 = 1681.974450955533;
        let q = 0.7071752369554196;
        let gain_db = 3.999843853973347;
        let k = (PI * f0 / fs).tan();
        let vh = 10f64.powf(gain_db / 20.0);
        let vb = vh.powf(0.4996667741545416);
        let a0 = 1.0 + k / q + k * k;
        Self {
            b0: (vh + vb * k / q + k * k) / a0,
            b1: 2.0 * (k * k - vh) / a0,
            b2: (vh - vb * k / q + k * k) / a0,
            a1: 2.0 * (k * k - 1.0) / a0,
            a2: (1.0 - k / q + k * k) / a0,
        }
    }

    /// BS.1770 stage 2: high-pass "RLB" filter.
    fn k_weighting_highpass(fs: f64) -> Self {
        let f0 = 38.13547087602444;
        let q = 0.5003270373238773;
        let k = (PI * f0 / fs).tan();
        let a0 = 1.0 + k / q + k * k;
        Self {
            b0: 1.0,
            b1: -2.0,
            b2: 1.0,
            a1: 2.0 * (k * k - 1.0) / a0,
            a2: (1.0 - k / q + k * k) / a0,
        }
    }

    /// Filter one channel in place (Direct Form I).
    fn process(&self, samples: &mut [f64]) {
        let (mut x1, mut x2, mut y1, mut y2) = (0.0, 0.0, 0.0, 0.0);
        for sample in samples.iter_mut() {
            let x0 = *sample;
            let y0 = self.b0 * x0 + self.b1 * x1 + self.b2 * x2 - self.a1 * y1 - self.a2 * y2;
            x2 = x1;
            x1 = x0;
            y2 = y1;
            y1 = y0;
            *sample = y0;
        }
    }
}

pub(crate) fn analyze_i16(samples: &[i16], sample_rate: u32, channels: u16) -> LoudnessStats {
    let floats: Vec<f32> = samples.iter().map(|s| f32::from(*s) / 32768.0).collect();
    analyze_f32(&floats, sample_rate, channels)
}

pub(crate) fn analyze_f32(samples: &[f32], sample_rate: u32, channels: u16) -> LoudnessStats {
    let channels = usize::from(channels.max(1));
    let fs = f64::from(sample_rate.max(1));
    LoudnessStats {
        loudness_lufs: integrated_lufs(samples, fs, channels),
        true_peak_dbfs: true_peak_dbfs(samples, channels),
    }
}

/// Split interleaved samples into per-channel planes of `f64`.
fn deinterleave(samples: &[f32], channels: usize) -> Vec<Vec<f64>> {
    let frames = samples.len() / channels;
    let mut planes = vec![Vec::with_capacity(frames); channels];
    for frame in 0..frames {
        for (ch, plane) in planes.iter_mut().enumerate() {
            plane.push(f64::from(samples[frame * channels + ch]));
        }
    }
    planes
}

fn integrated_lufs(samples: &[f32], fs: f64, channels: usize) -> f32 {
    if samples.is_empty() {
        return SILENCE_FLOOR_LUFS;
    }

    let shelf = Biquad::k_weighting_shelf(fs);
    let highpass = Biquad::k_weighting_highpass(fs);
    let mut planes = deinterleave(samples, channels);
    for plane in &mut planes {
        shelf.process(plane);
        highpass.process(plane);
    }
    let frames = planes.first().map_or(0, Vec::len);
    if frames == 0 {
        return SILENCE_FLOOR_LUFS;
    }

    // 400 ms blocks with 75% overlap (100 ms hop).
    let block = ((fs * 0.4).round() as usize).max(1);
    let hop = ((fs * 0.1).round() as usize).max(1);

    // Per-block summed-channel mean square: sum_c G_c * z_c, with G_c = 1.0 for
    // mono and for L/R stereo (the only layouts produced here).
    let mut block_powers: Vec<f64> = Vec::new();
    if frames < block {
        block_powers.push(block_power(&planes, 0, frames));
    } else {
        let mut start = 0;
        while start + block <= frames {
            block_powers.push(block_power(&planes, start, block));
            start += hop;
        }
    }

    // Absolute gate at −70 LUFS (block loudness = −0.691 + 10·log10(power)).
    let abs_gated: Vec<f64> = block_powers
        .into_iter()
        .filter(|p| *p > 0.0 && block_loudness(*p) >= -70.0)
        .collect();
    if abs_gated.is_empty() {
        return SILENCE_FLOOR_LUFS;
    }

    // Relative gate: −10 LU below the mean power of the absolute-gated blocks.
    let mean_abs = abs_gated.iter().sum::<f64>() / abs_gated.len() as f64;
    let rel_threshold = block_loudness(mean_abs) - 10.0;
    let rel_gated: Vec<f64> = abs_gated
        .iter()
        .copied()
        .filter(|p| block_loudness(*p) >= rel_threshold)
        .collect();
    let gated = if rel_gated.is_empty() {
        abs_gated
    } else {
        rel_gated
    };

    let mean_power = gated.iter().sum::<f64>() / gated.len() as f64;
    if mean_power <= 0.0 {
        return SILENCE_FLOOR_LUFS;
    }
    (block_loudness(mean_power) as f32).max(SILENCE_FLOOR_LUFS)
}

fn block_loudness(power: f64) -> f64 {
    -0.691 + 10.0 * power.log10()
}

/// Sum over channels of the mean square over `[start, start + len)`.
fn block_power(planes: &[Vec<f64>], start: usize, len: usize) -> f64 {
    let mut total = 0.0;
    for plane in planes {
        let end = (start + len).min(plane.len());
        if end <= start {
            continue;
        }
        let sum_sq: f64 = plane[start..end].iter().map(|s| s * s).sum();
        total += sum_sq / (end - start) as f64;
    }
    total
}

/// 4× oversampled true peak in dBFS. The raw samples are always included so the
/// result is never below the sample peak; the interpolated inter-sample phases
/// catch overshoot a plain sample-peak meter misses.
fn true_peak_dbfs(samples: &[f32], channels: usize) -> f32 {
    const OVERSAMPLE: usize = 4;
    const TAPS: usize = 12;
    let planes = deinterleave(samples, channels);
    let phases = polyphase_kernel(OVERSAMPLE, TAPS);
    let mut peak = 1e-7f64;
    for plane in &planes {
        let n = plane.len();
        for &s in plane {
            peak = peak.max(s.abs());
        }
        for i in 0..n {
            for phase in &phases[1..] {
                let mut acc = 0.0;
                for (t, coeff) in phase.iter().enumerate() {
                    let idx = i as isize + t as isize - (TAPS as isize / 2 - 1);
                    if idx >= 0 && (idx as usize) < n {
                        acc += plane[idx as usize] * coeff;
                    }
                }
                peak = peak.max(acc.abs());
            }
        }
    }
    (20.0 * peak.log10()) as f32
}

/// `m` polyphase sub-filters (each `taps` long) of a Hann-windowed-sinc
/// low-pass for `m`× interpolation. Each phase is normalized to unit DC gain so
/// a constant input maps to itself.
fn polyphase_kernel(m: usize, taps: usize) -> Vec<Vec<f64>> {
    let length = m * taps;
    let center = (length as f64 - 1.0) / 2.0;
    let mut phases = vec![vec![0.0f64; taps]; m];
    for (phase, sub) in phases.iter_mut().enumerate() {
        for (t, coeff) in sub.iter_mut().enumerate() {
            let n = (phase + t * m) as f64;
            let x = (n - center) / m as f64;
            let sinc = if x.abs() < 1e-9 {
                1.0
            } else {
                (PI * x).sin() / (PI * x)
            };
            let window = 0.5 - 0.5 * (2.0 * PI * n / (length as f64 - 1.0)).cos();
            *coeff = sinc * window;
        }
        let sum: f64 = sub.iter().sum();
        if sum.abs() > 1e-12 {
            for coeff in sub.iter_mut() {
                *coeff /= sum;
            }
        }
    }
    phases
}

#[cfg(test)]
mod tests {
    use super::{analyze_f32, analyze_i16, SILENCE_FLOOR_LUFS};
    use std::f64::consts::TAU;

    fn sine(freq: f64, amp: f64, secs: f64, fs: u32) -> Vec<f32> {
        let n = (secs * f64::from(fs)) as usize;
        (0..n)
            .map(|i| (amp * (TAU * freq * i as f64 / f64::from(fs)).sin()) as f32)
            .collect()
    }

    #[test]
    fn sine_loudness_is_a_plausible_lufs_value() {
        // 1 kHz sine at −6 dBFS (amp 0.5). Real K-weighted integrated loudness
        // lands in the single-digit-negative LUFS range; assert a generous band
        // (the point is that it is a real, finite LUFS, not garbage/NaN).
        let stats = analyze_f32(&sine(1000.0, 0.5, 3.0, 48_000), 48_000, 1);
        assert!(
            (-12.0..=-6.0).contains(&stats.loudness_lufs),
            "unexpected loudness {}",
            stats.loudness_lufs
        );
    }

    #[test]
    fn k_weighting_attenuates_low_frequencies() {
        // Equal-amplitude tones: K-weighting makes a 60 Hz tone read clearly
        // quieter than a 1 kHz tone — the old plain-RMS measure rated them equal
        // (both ≈ −9 dBFS). The RLB high-pass attenuates 60 Hz by ~3.6 LU here.
        let low = analyze_f32(&sine(60.0, 0.5, 3.0, 48_000), 48_000, 1).loudness_lufs;
        let mid = analyze_f32(&sine(1000.0, 0.5, 3.0, 48_000), 48_000, 1).loudness_lufs;
        assert!(
            low < mid - 3.0,
            "expected 60 Hz ({low}) >3 LU quieter than 1 kHz ({mid})"
        );
    }

    #[test]
    fn absolute_gate_excludes_silence() {
        // 1 s loud tone then 4 s of near-silence: the −70 LUFS absolute gate must
        // exclude the quiet tail, so the integrated value tracks the loud part.
        let loud = sine(1000.0, 0.5, 1.0, 48_000);
        let mut padded = loud.clone();
        padded.extend(std::iter::repeat_n(0.0f32, 4 * 48_000));
        let loud_only = analyze_f32(&loud, 48_000, 1).loudness_lufs;
        let gated = analyze_f32(&padded, 48_000, 1).loudness_lufs;
        assert!(
            (gated - loud_only).abs() < 1.5,
            "gated {gated} should track loud-only {loud_only}"
        );
    }

    #[test]
    fn true_peak_exceeds_sample_peak_on_intersample_overshoot() {
        // A full-scale tone at fs/4 with a 45° phase lands every sample at
        // ±0.707 (sample peak ≈ −3 dBFS) while the continuous peak is ~0 dBFS.
        let fs = 48_000u32;
        let n = fs as usize;
        let samples: Vec<f32> = (0..n)
            .map(|i| {
                (TAU * f64::from(fs) / 4.0 * i as f64 / f64::from(fs) + TAU / 8.0).sin() as f32
            })
            .collect();
        let sample_peak = samples.iter().fold(0.0f32, |m, s| m.max(s.abs()));
        let sample_peak_dbfs = 20.0 * sample_peak.log10();
        let tp = analyze_f32(&samples, fs, 1).true_peak_dbfs;
        assert!(
            tp > sample_peak_dbfs + 1.0,
            "true peak {tp} should exceed sample peak {sample_peak_dbfs} by >1 dB"
        );
    }

    #[test]
    fn silence_returns_finite_floor() {
        let stats = analyze_f32(&vec![0.0f32; 48_000], 48_000, 1);
        assert_eq!(stats.loudness_lufs, SILENCE_FLOOR_LUFS);
        assert!(stats.true_peak_dbfs.is_finite());
    }

    #[test]
    fn i16_and_f32_paths_agree() {
        let floats = sine(1000.0, 0.5, 2.0, 48_000);
        let ints: Vec<i16> = floats.iter().map(|s| (s * 32768.0) as i16).collect();
        let a = analyze_f32(&floats, 48_000, 1).loudness_lufs;
        let b = analyze_i16(&ints, 48_000, 1).loudness_lufs;
        assert!((a - b).abs() < 0.1, "f32 {a} vs i16 {b}");
    }

    #[test]
    fn analysis_is_deterministic() {
        let s = sine(440.0, 0.4, 2.0, 44_100);
        let a = analyze_f32(&s, 44_100, 2);
        let b = analyze_f32(&s, 44_100, 2);
        assert_eq!(a.loudness_lufs, b.loudness_lufs);
        assert_eq!(a.true_peak_dbfs, b.true_peak_dbfs);
    }
}
