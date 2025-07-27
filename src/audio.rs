use rodio::{Sink, Source};
use std::f32::consts::PI;
use std::sync::{Arc, Mutex};
use std::time::Duration;

pub struct AudioManager {
    pub sink: Option<Arc<Mutex<Sink>>>,
}

impl AudioManager {
    pub fn play_work_complete_sound(&self) {
        let tones = [
            (1760.0, Duration::from_millis(100)),
            (880.0, Duration::from_millis(100)),
            (440.0, Duration::from_millis(150)),
            (220.0, Duration::from_millis(200)),
        ];
        self.play_audio(&tones);
    }

    pub fn play_break_complete_sound(&self) {
        let tones = [
            (220.0, Duration::from_millis(150)),
            (440.0, Duration::from_millis(150)),
            (880.0, Duration::from_millis(150)),
            (1760.0, Duration::from_millis(300)),
        ];
        self.play_audio(&tones);
    }

    fn play_audio(&self, tones: &[(f32, Duration)]) {
        if let Some(ref sink) = self.sink {
            let sink = sink.lock().unwrap();
            let sample_rate = 44100;

            for (freq, dur) in tones {
                if *freq == 0.0 {
                    let silence = rodio::source::Zero::<f32>::new(1, sample_rate)
                        .take_duration(*dur)
                        .buffered();
                    sink.append(silence);
                } else {
                    let source = SquareWaveWithDecay::new(*freq, *dur, sample_rate);
                    sink.append(source);
                }
            }
        }
    }
}

struct SquareWaveWithDecay {
    freq: f32,
    duration: Duration,
    sample_rate: u32,
    sample_idx: usize,
    total_samples: usize,
}

impl SquareWaveWithDecay {
    fn new(freq: f32, duration: Duration, sample_rate: u32) -> Self {
        let total_samples = (duration.as_secs_f32() * sample_rate as f32) as usize;
        Self {
            freq,
            duration,
            sample_rate,
            sample_idx: 0,
            total_samples,
        }
    }
}

impl Iterator for SquareWaveWithDecay {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        if self.sample_idx >= self.total_samples {
            return None;
        }

        let t = self.sample_idx as f32 / self.sample_rate as f32;
        let phase = 2.0 * PI * self.freq * t;

        let wave = (1.0 * (phase).sin() + 1.0 / 3.0 * (3.0 * phase).sin() + 1.0 / 5.0 * (5.0 * phase).sin()) * (4.0 / PI);

        let env = 0.3 * (-20.0 * t).exp();
        let tail_start = self.duration.as_secs_f32() - 0.005;
        let fade = if t >= tail_start {
            let x = (self.duration.as_secs_f32() - t) / 0.005;
            x.clamp(0.0, 1.0)
        } else {
            1.0
        };

        let sample = wave as f32 * env * fade;
        self.sample_idx += 1;
        Some(sample)
    }
}

impl Source for SquareWaveWithDecay {
    fn current_frame_len(&self) -> Option<usize> {
        Some(self.total_samples - self.sample_idx)
    }

    fn channels(&self) -> u16 {
        1
    }

    fn sample_rate(&self) -> u32 {
        self.sample_rate
    }

    fn total_duration(&self) -> Option<Duration> {
        Some(self.duration)
    }
}