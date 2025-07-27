use rodio::Source;
use std::f32::consts::PI;
use std::time::Duration;

pub struct AudioManager {
    // No need to store sink anymore since we create fresh ones for each playback
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

    pub fn play_break_complete_music(&self) {
        // Play notification + longer melody as one continuous sequence
        let complete_sequence = [
            // Initial notification tones
            (220.0, Duration::from_millis(150)),
            (440.0, Duration::from_millis(150)),
            (880.0, Duration::from_millis(150)),
            (1760.0, Duration::from_millis(300)),
            (0.0, Duration::from_millis(300)), // Pause between notification and melody
            // Phrase 1 - Gentle wake-up call
            (523.25, Duration::from_millis(300)), // C5
            (587.33, Duration::from_millis(300)), // D5
            (659.25, Duration::from_millis(300)), // E5
            (698.46, Duration::from_millis(400)), // F5
            (0.0, Duration::from_millis(100)),    // Rest
            // Phrase 2 - Building energy
            (783.99, Duration::from_millis(300)),  // G5
            (880.00, Duration::from_millis(300)),  // A5
            (987.77, Duration::from_millis(300)),  // B5
            (1046.50, Duration::from_millis(500)), // C6
            (0.0, Duration::from_millis(200)),     // Rest
            // Phrase 3 - Descending comfort
            (1046.50, Duration::from_millis(250)), // C6
            (987.77, Duration::from_millis(250)),  // B5
            (880.00, Duration::from_millis(250)),  // A5
            (783.99, Duration::from_millis(250)),  // G5
            (698.46, Duration::from_millis(300)),  // F5
            (659.25, Duration::from_millis(400)),  // E5
            (0.0, Duration::from_millis(150)),     // Rest
            // Phrase 4 - Motivational ending
            (523.25, Duration::from_millis(200)),  // C5
            (659.25, Duration::from_millis(200)),  // E5
            (783.99, Duration::from_millis(200)),  // G5
            (1046.50, Duration::from_millis(300)), // C6
            (1174.66, Duration::from_millis(200)), // D6
            (1318.51, Duration::from_millis(600)), // E6 - Final note
        ];
        self.play_audio(&complete_sequence);
    }

    fn play_audio(&self, tones: &[(f32, Duration)]) {
        // Create a new stream and sink for each audio playback
        if let Ok((_stream, stream_handle)) = rodio::OutputStream::try_default() {
            if let Ok(sink) = rodio::Sink::try_new(&stream_handle) {
                let sample_rate = 44100;

                for (freq, dur) in tones {
                    if *freq == 0.0 {
                        let silence = rodio::source::Zero::<f32>::new(1, sample_rate).take_duration(*dur).buffered();
                        sink.append(silence);
                    } else {
                        let source = SquareWaveWithDecay::new(*freq, *dur, sample_rate);
                        sink.append(source);
                    }
                }

                // Wait for the audio to finish playing
                sink.sleep_until_end();
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
