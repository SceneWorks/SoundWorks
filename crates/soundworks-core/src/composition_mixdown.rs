//! Offline composition mixdown (UX-NB1).
//!
//! The Multitrack "Render Mixdown" needs a real engine that sums the
//! composition's per-track clips into a single rendered asset — distinct from the
//! single-clip generative adapters in `runtime`. This module is the pure mixing
//! core: given resolved clip PCM plus per-clip gain / pan / fade / placement and a
//! master gain, it produces an interleaved output buffer. It does no I/O, so the
//! mixing math (gain, overlap, clipping, fades, sample-rate conversion) is unit
//! tested in isolation; `runtime` resolves clip PCM from the library and wraps
//! this in the F-006 worker + cancellation.

/// Convert a decibel gain to a linear multiplier.
pub fn db_to_linear(db: f32) -> f32 {
    10f32.powf(db / 20.0)
}

/// One source clip placed on the timeline, with its PCM already resolved to
/// interleaved `f32` samples in [-1.0, 1.0].
#[derive(Debug, Clone)]
pub struct MixClip {
    pub samples: Vec<f32>,
    pub source_channels: u16,
    pub source_sample_rate: u32,
    /// Where the clip begins on the output timeline.
    pub timeline_start_ms: u64,
    /// The window of the source used (relative to the source's own start).
    pub source_start_ms: u64,
    pub source_end_ms: u64,
    /// Linear gain (track gain * clip gain already combined by the caller).
    pub gain: f32,
    /// -1.0 = hard left, 0.0 = center, +1.0 = hard right.
    pub pan: f32,
    pub fade_in_ms: u64,
    pub fade_out_ms: u64,
}

/// A full mixdown request: output spec + the clips to sum.
#[derive(Debug, Clone)]
pub struct MixRequest {
    pub sample_rate: u32,
    pub channels: u16,
    pub duration_ms: u64,
    pub master_gain: f32,
    pub clips: Vec<MixClip>,
}

fn sample_source(clip: &MixClip, frame: usize, channel: usize) -> f32 {
    // Read one (interpolated) source sample for the given source frame, folding
    // channels: a mono source feeds both output channels; a stereo source maps
    // L/R, with any extra output channel reusing channel 0.
    let src_channels = clip.source_channels.max(1) as usize;
    let total_frames = clip.samples.len() / src_channels;
    if total_frames == 0 {
        return 0.0;
    }
    let src_channel = channel.min(src_channels - 1);
    let index = frame.min(total_frames - 1) * src_channels + src_channel;
    clip.samples.get(index).copied().unwrap_or(0.0)
}

fn fade_scalar(position_ms: f64, clip_len_ms: f64, fade_in_ms: f64, fade_out_ms: f64) -> f32 {
    let mut scale = 1.0f64;
    if fade_in_ms > 0.0 && position_ms < fade_in_ms {
        scale = scale.min(position_ms / fade_in_ms);
    }
    if fade_out_ms > 0.0 {
        let from_end = clip_len_ms - position_ms;
        if from_end < fade_out_ms {
            scale = scale.min((from_end / fade_out_ms).max(0.0));
        }
    }
    scale.clamp(0.0, 1.0) as f32
}

/// Mix the request into an interleaved output buffer, clamped to [-1.0, 1.0].
pub fn mix(request: &MixRequest) -> Vec<f32> {
    let channels = request.channels.max(1) as usize;
    let out_sr = request.sample_rate.max(1);
    let out_frames =
        ((request.duration_ms as u128 * out_sr as u128) / 1000) as usize;
    let mut out = vec![0.0f32; out_frames * channels];
    if out_frames == 0 {
        return out;
    }

    for clip in &request.clips {
        if clip.source_end_ms <= clip.source_start_ms || clip.samples.is_empty() {
            continue;
        }
        let clip_len_ms = (clip.source_end_ms - clip.source_start_ms) as f64;
        let clip_out_frames = ((clip_len_ms * out_sr as f64) / 1000.0).round() as usize;
        let start_frame = ((clip.timeline_start_ms as f64 * out_sr as f64) / 1000.0)
            .round() as usize;
        // Pan: simple linear-taper so center keeps both channels at unity and a
        // hard pan silences the opposite channel.
        let pan = clip.pan.clamp(-1.0, 1.0);
        let left_pan = if pan > 0.0 { 1.0 - pan } else { 1.0 };
        let right_pan = if pan < 0.0 { 1.0 + pan } else { 1.0 };
        let ratio = clip.source_sample_rate.max(1) as f64 / out_sr as f64;
        let source_start_frame = (clip.source_start_ms as f64 * clip.source_sample_rate as f64)
            / 1000.0;

        for i in 0..clip_out_frames {
            let out_frame = start_frame + i;
            if out_frame >= out_frames {
                break;
            }
            let position_ms = (i as f64 / out_sr as f64) * 1000.0;
            let env = fade_scalar(
                position_ms,
                clip_len_ms,
                clip.fade_in_ms as f64,
                clip.fade_out_ms as f64,
            );
            let gain = clip.gain * env;
            let src_frame = (source_start_frame + i as f64 * ratio).round() as usize;
            for ch in 0..channels {
                let raw = sample_source(clip, src_frame, ch);
                let pan_gain = if channels >= 2 {
                    if ch == 0 {
                        left_pan
                    } else if ch == 1 {
                        right_pan
                    } else {
                        1.0
                    }
                } else {
                    1.0
                };
                out[out_frame * channels + ch] += raw * gain * pan_gain;
            }
        }
    }

    for sample in out.iter_mut() {
        *sample = (*sample * request.master_gain).clamp(-1.0, 1.0);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn const_clip(value: f32, frames: usize, gain: f32) -> MixClip {
        MixClip {
            samples: vec![value; frames],
            source_channels: 1,
            source_sample_rate: 48_000,
            timeline_start_ms: 0,
            source_start_ms: 0,
            source_end_ms: (frames as u64 * 1000) / 48_000,
            gain,
            pan: 0.0,
            fade_in_ms: 0,
            fade_out_ms: 0,
        }
    }

    #[test]
    fn db_to_linear_unity_and_minus_six() {
        assert!((db_to_linear(0.0) - 1.0).abs() < 1e-6);
        assert!((db_to_linear(-6.0) - 0.5012).abs() < 1e-3);
    }

    #[test]
    fn applies_clip_and_master_gain() {
        let request = MixRequest {
            sample_rate: 48_000,
            channels: 1,
            duration_ms: 100,
            master_gain: 1.0,
            clips: vec![const_clip(0.25, 4_800, 2.0)],
        };
        let out = mix(&request);
        // 0.25 * gain 2.0 = 0.5, master 1.0.
        assert!((out[10] - 0.5).abs() < 1e-4, "got {}", out[10]);
    }

    #[test]
    fn overlapping_clips_sum_and_clip() {
        // Two full-scale clips overlap -> sum 2.0 clamped to 1.0.
        let request = MixRequest {
            sample_rate: 48_000,
            channels: 1,
            duration_ms: 100,
            master_gain: 1.0,
            clips: vec![const_clip(1.0, 4_800, 1.0), const_clip(1.0, 4_800, 1.0)],
        };
        let out = mix(&request);
        assert!((out[100] - 1.0).abs() < 1e-6, "expected clamp to 1.0");
    }

    #[test]
    fn hard_pan_silences_opposite_channel() {
        let mut clip = const_clip(0.5, 4_800, 1.0);
        clip.pan = 1.0; // hard right
        let request = MixRequest {
            sample_rate: 48_000,
            channels: 2,
            duration_ms: 100,
            master_gain: 1.0,
            clips: vec![clip],
        };
        let out = mix(&request);
        // frame 50: left (idx 100) silent, right (idx 101) ~0.5
        assert!(out[100].abs() < 1e-6, "left should be silent: {}", out[100]);
        assert!((out[101] - 0.5).abs() < 1e-4, "right: {}", out[101]);
    }

    #[test]
    fn fade_in_ramps_from_zero() {
        let mut clip = const_clip(1.0, 4_800, 1.0);
        clip.fade_in_ms = 50;
        let request = MixRequest {
            sample_rate: 48_000,
            channels: 1,
            duration_ms: 100,
            master_gain: 1.0,
            clips: vec![clip],
        };
        let out = mix(&request);
        // first sample near zero, midpoint of fade ~0.5, after fade full.
        assert!(out[0].abs() < 0.05, "fade start: {}", out[0]);
        assert!(out[4_000] > 0.9, "post-fade: {}", out[4_000]);
    }

    #[test]
    fn timeline_offset_leaves_leading_silence() {
        let mut clip = const_clip(1.0, 4_800, 1.0);
        clip.timeline_start_ms = 50;
        let request = MixRequest {
            sample_rate: 48_000,
            channels: 1,
            duration_ms: 200,
            master_gain: 1.0,
            clips: vec![clip],
        };
        let out = mix(&request);
        assert!(out[10].abs() < 1e-6, "leading silence before offset");
        assert!((out[2_500] - 1.0).abs() < 1e-4, "clip audible after offset");
    }
}
